use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, BinaryHeap},
    time::SystemTime,
};

use maplit::btreeset;
use ruma::{
    events::EventType,
    identifiers::{EventId, RoomId, RoomVersionId},
};
use serde::{Deserialize, Serialize};

mod event_auth;
mod room_version;
mod state_event;
mod state_store;

pub use event_auth::{auth_check, auth_types_for_event};
pub use state_event::StateEvent;
pub use state_store::StateStore;

// We want to yield to the reactor occasionally during state res when dealing
// with large data sets, so that we don't exhaust the reactor. This is done by
// yielding to reactor during loops every N iterations.
const _YIELD_AFTER_ITERATIONS: usize = 100;

pub enum ResolutionResult {
    Conflicted(Vec<StateMap<EventId>>),
    Resolved(StateMap<EventId>),
}

/// A mapping of event type and state_key to some value `T`, usually an `EventId`.
pub type StateMap<T> = BTreeMap<(EventType, String), T>;

/// A mapping of `EventId` to `T`, usually a `StateEvent`.
pub type EventMap<T> = BTreeMap<EventId, T>;

#[derive(Debug, Default, Deserialize, Serialize)] // TODO make the ser/de impls useful
pub struct StateResolution {
    // TODO remove pub after initial testing
    /// The set of resolved events over time.
    pub resolved_events: Vec<StateEvent>,
    /// The resolved state, kept to have easy access to the last resolved
    /// layer of state.
    pub state: BTreeMap<EventType, BTreeMap<String, StateEvent>>,
    /// The graph of authenticated events, kept to find the most recent auth event
    /// in a chain for incoming state sets.
    pub auth_graph: BTreeMap<EventId, Vec<StateMap<EventId>>>,
    /// The last known point in the state graph.
    pub most_recent_resolved: Option<(EventType, String)>,

    // fields for temp storage during resolution
    pub conflicting_events: Vec<StateEvent>,
}

impl StateResolution {
    /// Resolve sets of state events as they come in. Internally `StateResolution` builds a graph
    /// and an auth chain to allow for state conflict resolution.
    pub fn resolve(
        &mut self,
        room_id: &RoomId,
        room_version: &RoomVersionId,
        state_sets: &[StateMap<EventId>],
        // TODO remove or make this mut so we aren't cloning the whole thing
        event_map: Option<EventMap<StateEvent>>,
        store: &dyn StateStore,
        // TODO actual error handling (`thiserror`??)
    ) -> Result<ResolutionResult, String> {
        tracing::info!("State resolution starting");

        let mut event_map = if let Some(ev_map) = event_map {
            ev_map
        } else {
            EventMap::new()
        };
        // split non-conflicting and conflicting state
        let (clean, conflicting) = self.separate(&state_sets);

        if conflicting.is_empty() {
            return Ok(ResolutionResult::Resolved(clean));
        }

        tracing::info!("computing {} conflicting events", conflicting.len());

        // the set of auth events that are not common across server forks
        let mut auth_diff = self.get_auth_chain_diff(room_id, &state_sets, &event_map, store)?;

        // add the auth_diff to conflicting now we have a full set of conflicting events
        auth_diff.extend(conflicting.values().cloned().flatten());
        let mut all_conflicted = auth_diff
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        tracing::debug!(
            "FULL CONF {:?}",
            all_conflicted
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );

        tracing::info!("full conflicted set is {} events", all_conflicted.len());

        // gather missing events for the event_map
        let events = store
            .get_events(
                &all_conflicted
                    .iter()
                    // we only want the events we don't know about yet
                    .filter(|id| !event_map.contains_key(id))
                    .cloned()
                    .collect::<Vec<_>>(),
            )
            .unwrap();

        // update event_map to include the fetched events
        event_map.extend(
            events
                .into_iter()
                .flat_map(|ev| Some((ev.event_id()?.clone(), ev))),
        );

        for event in event_map.values() {
            if event.room_id() != Some(room_id) {
                return Err(format!(
                    "resolving event {} in room {}, when correct room is {}",
                    event
                        .event_id()
                        .map(|id| id.as_str())
                        .unwrap_or("`unknown`"),
                    event.room_id().map(|id| id.as_str()).unwrap_or("`unknown`"),
                    room_id.as_str()
                ));
            }
        }

        // TODO make sure each conflicting event is in event_map??
        // synapse says `full_set = {eid for eid in full_conflicted_set if eid in event_map}`
        all_conflicted.retain(|id| event_map.contains_key(id));
        println!(
            "ALL {:?}",
            all_conflicted
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );
        // get only the power events with a state_key: "" or ban/kick event (sender != state_key)
        let power_events = all_conflicted
            .iter()
            .filter(|id| is_power_event(id, store))
            .cloned()
            .collect::<Vec<_>>();

        println!(
            "POWER {:?}",
            power_events
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );

        // sort the power events based on power_level/clock/event_id and outgoing/incoming edges
        let mut sorted_power_levels = self.reverse_topological_power_sort(
            room_id,
            &power_events,
            &mut event_map,
            store,
            &all_conflicted,
        );

        println!(
            "SRTD {:?}",
            sorted_power_levels
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );

        // sequentially auth check each power level event event.
        let resolved = self.iterative_auth_check(
            room_id,
            room_version,
            &sorted_power_levels,
            &clean,
            &mut event_map,
            store,
        )?;

        tracing::debug!(
            "AUTHED {:?}",
            resolved
                .iter()
                .map(|(key, id)| (key, id.to_string()))
                .collect::<Vec<_>>()
        );

        // At this point the power_events have been resolved we now have to
        // sort the remaining events using the mainline of the resolved power level.
        sorted_power_levels.dedup();
        let deduped_power_ev = sorted_power_levels;

        // we have resolved the power events so remove them, I'm sure theres other reasons to do so
        let events_to_resolve = all_conflicted
            .iter()
            .filter(|id| !deduped_power_ev.contains(id))
            .cloned()
            .collect::<Vec<_>>();

        println!(
            "LEFT {:?}",
            events_to_resolve
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );

        let power_event = resolved.get(&(EventType::RoomPowerLevels, "".into()));

        tracing::debug!("PL {:?}", power_event);

        let sorted_left_events =
            self.mainline_sort(room_id, &events_to_resolve, power_event, &event_map, store);

        println!(
            "SORTED LEFT {:?}",
            sorted_left_events
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );

        let mut resolved_state = self.iterative_auth_check(
            room_id,
            room_version,
            &sorted_left_events,
            &resolved,
            &mut event_map,
            store,
        )?;

        // add unconflicted state to the resolved state
        resolved_state.extend(clean);

        // TODO return something not a place holder
        Ok(ResolutionResult::Resolved(resolved_state))
    }

    /// Split the events that have no conflicts from those that are conflicting.
    ///
    /// The tuple looks like `(unconflicted, conflicted)`.
    pub fn separate(
        &mut self,
        state_sets: &[StateMap<EventId>],
    ) -> (StateMap<EventId>, StateMap<Vec<EventId>>) {
        use itertools::Itertools;

        tracing::info!(
            "seperating {} sets of events into conflicted/unconflicted",
            state_sets.len()
        );

        let mut unconflicted_state = StateMap::new();
        let mut conflicted_state = StateMap::new();

        for key in state_sets.iter().flat_map(|map| map.keys()) {
            let mut event_ids = state_sets
                .iter()
                .flat_map(|map| map.get(key).cloned())
                .dedup()
                .collect::<Vec<EventId>>();

            if event_ids.len() == 1 {
                // unwrap is ok since we know the len is 1
                unconflicted_state.insert(key.clone(), event_ids.pop().unwrap());
            } else {
                conflicted_state.insert(key.clone(), event_ids);
            }
        }

        (unconflicted_state, conflicted_state)
    }

    /// Returns a Vec of deduped EventIds that appear in some chains but no others.
    pub fn get_auth_chain_diff(
        &mut self,
        room_id: &RoomId,
        state_sets: &[StateMap<EventId>],
        _event_map: &EventMap<StateEvent>,
        store: &dyn StateStore,
    ) -> Result<Vec<EventId>, String> {
        use itertools::Itertools;

        tracing::debug!("calculating auth chain difference");

        store.auth_chain_diff(
            room_id,
            state_sets
                .iter()
                .map(|map| map.values().cloned().collect())
                .dedup()
                .collect::<Vec<_>>(),
        )
    }

    pub fn reverse_topological_power_sort(
        &mut self,
        room_id: &RoomId,
        power_events: &[EventId],
        event_map: &EventMap<StateEvent>,
        store: &dyn StateStore,
        auth_diff: &[EventId],
    ) -> Vec<EventId> {
        tracing::debug!("reverse topological sort of power events");

        let mut graph = BTreeMap::new();
        for (idx, event_id) in power_events.iter().enumerate() {
            self.add_event_and_auth_chain_to_graph(room_id, &mut graph, event_id, store, auth_diff);

            // We yield occasionally when we're working with large data sets to
            // ensure that we don't block the reactor loop for too long.
            if idx % _YIELD_AFTER_ITERATIONS == 0 {
                // yield clock.sleep(0)
            }
        }

        // this is used in the `key_fn` passed to the lexico_topo_sort fn
        let mut event_to_pl = BTreeMap::new();
        for (idx, event_id) in graph.keys().enumerate() {
            let pl = self.get_power_level_for_sender(room_id, &event_id, event_map, store);
            println!("{} power level {}", event_id.to_string(), pl);

            event_to_pl.insert(event_id.clone(), pl);

            // We yield occasionally when we're working with large data sets to
            // ensure that we don't block the reactor loop for too long.
            if idx % _YIELD_AFTER_ITERATIONS == 0 {
                // yield clock.sleep(0)
            }
        }

        self.lexicographical_topological_sort(&mut graph, |event_id| {
            // tracing::debug!("{:?}", event_map.get(event_id).unwrap().origin_server_ts());
            let ev = event_map.get(event_id).unwrap();
            let pl = event_to_pl.get(event_id).unwrap();
            // This return value is the key used for sorting events,
            // events are then sorted by power level, time,
            // and lexically by event_id.
            (*pl, ev.origin_server_ts().clone(), ev.event_id().cloned())
        })
    }

    /// Sorts the event graph based on number of outgoing/incoming edges, where
    /// `key_fn` is used as a tie breaker. The tie breaker happens based on
    /// power level, age, and event_id.
    pub fn lexicographical_topological_sort<F>(
        &mut self,
        graph: &BTreeMap<EventId, Vec<EventId>>,
        key_fn: F,
    ) -> Vec<EventId>
    where
        F: Fn(&EventId) -> (i64, SystemTime, Option<EventId>),
    {
        tracing::info!("starting lexicographical topological sort");
        // NOTE: an event that has no incoming edges happened most recently,
        // and an event that has no outgoing edges happened least recently.

        // NOTE: this is basically Kahn's algorithm except we look at nodes with no
        // outgoing edges, c.f.
        // https://en.wikipedia.org/wiki/Topological_sorting#Kahn's_algorithm

        // TODO make the BTreeSet conversion cleaner ??
        let mut outdegree_map: BTreeMap<EventId, BTreeSet<EventId>> = graph
            .into_iter()
            .map(|(k, v)| (k.clone(), v.into_iter().cloned().collect()))
            .collect();
        let mut reverse_graph = BTreeMap::new();

        // Vec of nodes that have zero out degree, least recent events.
        let mut zero_outdegree = vec![];

        for (node, edges) in graph.iter() {
            if edges.is_empty() {
                // the `Reverse` is because rusts bin heap sorts largest -> smallest we need
                // smallest -> largest
                zero_outdegree.push(Reverse((key_fn(node), node)));
            }

            reverse_graph.entry(node).or_insert(btreeset![]);
            for edge in edges {
                reverse_graph
                    .entry(edge)
                    .or_insert(btreeset![])
                    .insert(node);
            }
        }

        let mut heap = BinaryHeap::from(zero_outdegree);

        // we remove the oldest node (most incoming edges) and check against all other
        let mut sorted = vec![];
        // match out the `Reverse` and take the smallest `node` each time
        while let Some(Reverse((_, node))) = heap.pop() {
            let node: &EventId = node;
            for parent in reverse_graph.get(node).unwrap() {
                // the number of outgoing edges this node has
                let out = outdegree_map.get_mut(parent).unwrap();

                // only push on the heap once older events have been cleared
                out.remove(node);
                if out.is_empty() {
                    heap.push(Reverse((key_fn(parent), parent)));
                }
            }

            // synapse yields we push then return the vec
            sorted.push(node.clone());
        }

        // tracing::debug!(
        //     "{:#?}",
        //     sorted.iter().map(ToString::to_string).collect::<Vec<_>>()
        // );
        sorted
    }

    fn get_power_level_for_sender(
        &self,
        room_id: &RoomId,
        event_id: &EventId,
        event_map: &EventMap<StateEvent>, // TODO use event_map over store ??
        store: &dyn StateStore,
    ) -> i64 {
        tracing::info!("fetch event senders ({}) power level", event_id.to_string());
        let event = event_map.get(event_id);
        let mut pl = None;
        // TODO store.auth_event_ids returns "self" with the event ids is this ok
        // event.auth_event_ids does not include its own event id ?
        for aid in event_map.get(event_id).unwrap().auth_event_ids() {
            println!("aid {}", aid.to_string());
            if let Some(aev) = event_map.get(&aid) {
                if aev.is_type_and_key(EventType::RoomPowerLevels, "") {
                    pl = Some(aev);
                    break;
                }
            }
        }

        if pl.is_none() {
            for aid in event_map.get(event_id).unwrap().auth_event_ids() {
                println!("aid NONE {}", aid.to_string());

                if let Some(aev) = event_map.get(&aid) {
                    if aev.is_type_and_key(EventType::RoomCreate, "") {
                        if let Ok(content) = aev
                            .deserialize_content::<ruma::events::room::create::CreateEventContent>()
                        {
                            if &content.creator == aev.sender() {
                                return 100;
                            }
                            break;
                        }
                    }
                }
            }
            return 0;
        }

        if let Some(content) = pl
            .map(|pl| {
                pl.deserialize_content::<ruma::events::room::power_levels::PowerLevelsEventContent>(
                )
                .ok()
            })
            .flatten()
        {
            if let Some(ev) = event {
                if let Some(user) = content.users.get(ev.sender()) {
                    return (*user).into();
                }
            }
            content.state_default.into()
        } else {
            0
        }
    }

    fn iterative_auth_check(
        &mut self,
        _room_id: &RoomId,
        room_version: &RoomVersionId,
        power_events: &[EventId],
        unconflicted_state: &StateMap<EventId>,
        _event_map: &EventMap<StateEvent>, // TODO use event_map over store ??
        store: &dyn StateStore,
    ) -> Result<StateMap<EventId>, String> {
        tracing::info!("starting iterative auth check");

        let mut resolved_state = unconflicted_state.clone();

        for (idx, event_id) in power_events.iter().enumerate() {
            let event = store.get_event(event_id).unwrap();

            let mut auth_events = BTreeMap::new();
            for aid in event.auth_event_ids() {
                if let Ok(ev) = store.get_event(&aid) {
                    // TODO what to do when no state_key is found ??
                    // TODO check "rejected_reason", I'm guessing this is redacted_because for ruma ??
                    auth_events.insert((ev.kind(), ev.state_key().unwrap()), ev);
                } else {
                    tracing::warn!("auth event id for {} is missing {}", aid, event_id);
                }
            }

            for key in event_auth::auth_types_for_event(&event) {
                if let Some(ev_id) = resolved_state.get(&key) {
                    // TODO synapse gets the event from the store then checks its not None
                    // then pulls the same `ev_id` event from the event_map??
                    if let Ok(event) = store.get_event(ev_id) {
                        auth_events.insert(key.clone(), event);
                    }
                }
            }

            tracing::debug!("event to check {:?}", event.event_id().unwrap().to_string());
            if !event_auth::auth_check(room_version, &event, auth_events)
                .ok_or("Auth check failed due to deserialization most likely".to_string())
                .unwrap()
            {
                // TODO synapse passes here on AuthError ??
                tracing::warn!("event {} failed the authentication", event_id.to_string());
            } else {
                // add event to resolved state map
                resolved_state.insert((event.kind(), event.state_key().unwrap()), event_id.clone());
            }

            // We yield occasionally when we're working with large data sets to
            // ensure that we don't block the reactor loop for too long.
            if idx % _YIELD_AFTER_ITERATIONS == 0 {
                // yield clock.sleep(0)
            }
        }
        Ok(resolved_state)
    }

    /// Returns the sorted `to_sort` list of `EventId`s based on a mainline sort using
    /// the `resolved_power_level`.
    fn mainline_sort(
        &mut self,
        room_id: &RoomId,
        to_sort: &[EventId],
        resolved_power_level: Option<&EventId>,
        event_map: &EventMap<StateEvent>,
        store: &dyn StateStore,
    ) -> Vec<EventId> {
        tracing::debug!("mainline sort of remaining events");
        // tracing::debug!(
        //     "{:?}",
        //     to_sort.iter().map(ToString::to_string).collect::<Vec<_>>()
        // );
        // There can be no EventId's to sort, bail.
        if to_sort.is_empty() {
            return vec![];
        }

        let mut mainline = vec![];
        let mut pl = resolved_power_level.cloned();
        let mut idx = 0;
        while let Some(p) = pl {
            mainline.push(p.clone());
            // We don't need the actual pl_ev here since we delegate to the store
            let auth_events = store.auth_event_ids(room_id, &[p]).unwrap();
            pl = None;
            for aid in auth_events {
                let ev = store.get_event(&aid).unwrap();
                if ev.is_type_and_key(EventType::RoomPowerLevels, "") {
                    pl = Some(aid);
                    break;
                }
            }
            // We yield occasionally when we're working with large data sets to
            // ensure that we don't block the reactor loop for too long.
            if idx != 0 && idx % _YIELD_AFTER_ITERATIONS == 0 {
                // yield clock.sleep(0)
            }
            idx += 1;
        }

        let mainline_map = mainline
            .iter()
            .rev()
            .enumerate()
            .map(|(idx, eid)| ((*eid).clone(), idx))
            .collect::<BTreeMap<_, _>>();
        let mut sort_event_ids = to_sort.to_vec();

        let mut order_map = BTreeMap::new();
        for (idx, ev_id) in to_sort.iter().enumerate() {
            let depth = self.get_mainline_depth(store.get_event(ev_id).ok(), &mainline_map, store);
            order_map.insert(
                ev_id,
                (
                    depth,
                    event_map.get(ev_id).map(|ev| ev.origin_server_ts()),
                    ev_id, // TODO should this be a &str to sort lexically??
                ),
            );

            // We yield occasionally when we're working with large data sets to
            // ensure that we don't block the reactor loop for too long.
            if idx % _YIELD_AFTER_ITERATIONS == 0 {
                // yield clock.sleep(0)
            }
        }

        // sort the event_ids by their depth, timestamp and EventId
        // unwrap is OK order map and sort_event_ids are from to_sort (the same Vec)
        sort_event_ids.sort_by_key(|sort_id| order_map.get(sort_id).unwrap());

        sort_event_ids
    }

    fn get_mainline_depth(
        &mut self,
        mut event: Option<StateEvent>,
        mainline_map: &EventMap<usize>,
        store: &dyn StateStore,
    ) -> usize {
        while let Some(sort_ev) = event {
            tracing::debug!(
                "mainline EVENT ID {}",
                sort_ev.event_id().unwrap().to_string()
            );
            if let Some(id) = sort_ev.event_id() {
                if let Some(depth) = mainline_map.get(id) {
                    return *depth;
                }
            }

            let auth_events = sort_ev.auth_event_ids();
            tracing::debug!(
                "mainline AUTH EV {:?}",
                auth_events
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            );

            event = None;

            for aid in auth_events {
                let aev = store.get_event(&aid).unwrap();
                if aev.is_type_and_key(EventType::RoomPowerLevels, "") {
                    event = Some(aev);
                    break;
                }
            }
        }
        // Did not find a power level event so we default to zero
        0
    }

    fn add_event_and_auth_chain_to_graph(
        &self,
        room_id: &RoomId,
        graph: &mut BTreeMap<EventId, Vec<EventId>>,
        event_id: &EventId,
        store: &dyn StateStore,
        auth_diff: &[EventId],
    ) {
        let mut state = vec![event_id.clone()];
        while !state.is_empty() {
            // we just checked if it was empty so unwrap is fine
            let eid = state.pop().unwrap();
            graph.entry(eid.clone()).or_insert(vec![]);
            // prefer the store to event as the store filters dedups the events
            // otherwise it seems we can loop forever
            for aid in store.auth_event_ids(room_id, &[eid.clone()]).unwrap() {
                if auth_diff.contains(&aid) {
                    if !graph.contains_key(&aid) {
                        state.push(aid.clone());
                    }

                    // we just inserted this at the start of the while loop
                    graph.get_mut(&eid).unwrap().push(aid);
                }
            }
        }
    }
}

pub fn is_power_event(event_id: &EventId, store: &dyn StateStore) -> bool {
    match store.get_event(event_id) {
        Ok(state) => state.is_power_event(),
        _ => false, // TODO this shouldn't eat errors?
    }
}
