use std::{
    collections::{BTreeMap, BinaryHeap},
    time::SystemTime,
};

use ruma::{
    events::EventType,
    identifiers::{EventId, RoomId, RoomVersionId},
};
use serde::{Deserialize, Serialize};

mod event_auth;
mod room_version;
mod state_event;
mod state_store;

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
        store: &dyn StateStore,
        // TODO actual error handling (`thiserror`??)
    ) -> Result<ResolutionResult, String> {
        tracing::debug!("State resolution starting");

        let mut event_map = EventMap::new();
        // split non-conflicting and conflicting state
        let (clean, conflicting) = self.separate(&state_sets);

        if conflicting.is_empty() {
            return Ok(ResolutionResult::Resolved(clean));
        }

        tracing::debug!("computing {} conflicting events", conflicting.len());

        // the set of auth events that are not common across server forks
        let mut auth_diff = self.get_auth_chain_diff(&state_sets, &mut event_map, store)?;

        // add the auth_diff to conflicting now we have a full set of conflicting events
        auth_diff.extend(conflicting.values().cloned().flatten());
        let mut all_conflicted = auth_diff;

        tracing::debug!("full conflicted set is {} events", all_conflicted.len());

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

        // get only the power events with a state_key: "" or ban/kick event (sender != state_key)
        let power_events = all_conflicted
            .iter()
            .filter(|id| is_power_event(id, store))
            .cloned()
            .collect::<Vec<_>>();

        // sort the power events based on power_level/clock/event_id and outgoing/incoming edges
        let mut sorted_power_levels = self.reverse_topological_power_sort(
            room_id,
            &power_events,
            &mut event_map,
            store,
            &all_conflicted,
        );

        // sequentially auth check each event.
        let resolved = self.iterative_auth_check(
            room_id,
            room_version,
            &sorted_power_levels,
            &clean,
            &mut event_map,
            store,
        )?;

        // At this point the power_events have been resolved we now have to
        // sort the remaining events using the mainline of the resolved power level.
        sorted_power_levels.dedup();
        let deduped_power_ev = sorted_power_levels;

        // we have resolved the power events so remove them, I'm sure theres other reasons to do so
        let events_to_resolve = all_conflicted
            .iter()
            .filter(|id| deduped_power_ev.contains(id))
            .cloned()
            .collect::<Vec<_>>();

        let power_event = resolved.get(&(EventType::RoomPowerLevels, "".into()));

        let sorted_left_events =
            self.mainline_sort(room_id, &events_to_resolve, power_event, &event_map, store);

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
    fn separate(
        &mut self,
        state_sets: &[StateMap<EventId>],
    ) -> (StateMap<EventId>, StateMap<Vec<EventId>>) {
        let mut unconflicted_state = StateMap::new();
        let mut conflicted_state = StateMap::new();

        for key in state_sets.iter().flat_map(|map| map.keys()) {
            let mut event_ids = state_sets
                .iter()
                .flat_map(|map| map.get(key).cloned())
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
    fn get_auth_chain_diff(
        &mut self,
        state_sets: &[StateMap<EventId>],
        _event_map: &EventMap<StateEvent>,
        store: &dyn StateStore,
    ) -> Result<Vec<EventId>, String> {
        use itertools::Itertools;

        tracing::debug!("calculating auth chain difference");
        store.auth_chain_diff(
            &state_sets
                .iter()
                .flat_map(|map| map.values())
                .dedup()
                .collect::<Vec<_>>(),
        )
    }

    fn reverse_topological_power_sort(
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

            event_to_pl.insert(event_id.clone(), pl);

            // We yield occasionally when we're working with large data sets to
            // ensure that we don't block the reactor loop for too long.
            if idx % _YIELD_AFTER_ITERATIONS == 0 {
                // yield clock.sleep(0)
            }
        }

        self.lexicographical_topological_sort(&mut graph, |event_id| {
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
    fn lexicographical_topological_sort<F>(
        &mut self,
        graph: &BTreeMap<EventId, Vec<EventId>>,
        key_fn: F,
    ) -> Vec<EventId>
    where
        F: Fn(&EventId) -> (i64, SystemTime, Option<EventId>),
    {
        // NOTE: an event that has no incoming edges happened most recently,
        // and an event that has no outgoing edges happened least recently.

        // NOTE: this is basically Kahn's algorithm except we look at nodes with no
        // outgoing edges, c.f.
        // https://en.wikipedia.org/wiki/Topological_sorting#Kahn's_algorithm
        let outdegree_map = graph;
        let mut reverse_graph = BTreeMap::new();

        // Vec of nodes that have zero out degree, least recent events.
        let mut zero_outdegree = vec![];

        for (node, edges) in graph.iter() {
            if edges.is_empty() {
                zero_outdegree.push((key_fn(node), node));
            }

            reverse_graph.insert(node, vec![]);
            for edge in edges {
                reverse_graph.entry(edge).or_insert(vec![]).push(node);
            }
        }

        let mut heap = BinaryHeap::from(zero_outdegree);

        // we remove the oldest node (most incoming edges) and check against all other
        //
        while let Some((_, node)) = heap.pop() {
            for parent in reverse_graph.get(node).unwrap() {
                let out = outdegree_map.get(parent).unwrap();
                if out.iter().filter(|id| *id == node).count() == 0 {
                    heap.push((key_fn(parent), parent));
                }
            }
        }

        // rust BinaryHeap does not iter in order so we gotta do it the long way
        let mut sorted = vec![];
        while let Some((_, id)) = heap.pop() {
            sorted.push(id.clone())
        }

        sorted
    }

    fn get_power_level_for_sender(
        &self,
        room_id: &RoomId,
        event_id: &EventId,
        _event_map: &EventMap<StateEvent>, // TODO use event_map over store ??
        store: &dyn StateStore,
    ) -> i64 {
        let mut pl = None;
        for aid in store.auth_event_ids(room_id, event_id).unwrap() {
            if let Ok(aev) = store.get_event(&aid) {
                if aev.is_type_and_key(EventType::RoomPowerLevels, "") {
                    pl = Some(aev);
                    break;
                }
            }
        }

        if pl.is_none() {
            for aid in store.auth_event_ids(room_id, event_id).unwrap() {
                if let Ok(aev) = store.get_event(&aid) {
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
            content.users_default.into()
        } else {
            0
        }
    }

    fn iterative_auth_check(
        &mut self,
        room_id: &RoomId,
        room_version: &RoomVersionId,
        power_events: &[EventId],
        unconflicted_state: &StateMap<EventId>,
        _event_map: &EventMap<StateEvent>, // TODO use event_map over store ??
        store: &dyn StateStore,
    ) -> Result<StateMap<EventId>, String> {
        tracing::debug!("starting iter auth check");
        let resolved_state = unconflicted_state.clone();
        for (idx, event_id) in power_events.iter().enumerate() {
            let event = store.get_event(event_id).unwrap();

            let mut auth_events = BTreeMap::new();
            for aid in store.auth_event_ids(room_id, event_id).unwrap() {
                if let Ok(ev) = store.get_event(&aid) {
                    // TODO is None the same as "" for state_key, pretty sure it is NOT
                    auth_events.insert((ev.kind(), ev.state_key().unwrap_or_default()), ev);
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

            if !event_auth::auth_check(room_version, &event, auth_events).ok_or("".to_string())? {}

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
            let auth_events = store.auth_event_ids(room_id, &p).unwrap();
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
            .enumerate()
            .map(|(idx, eid)| ((*eid).clone(), idx))
            .collect::<BTreeMap<_, _>>();
        let mut sort_event_ids = to_sort.to_vec();

        let mut order_map = BTreeMap::new();
        for (idx, ev_id) in to_sort.iter().enumerate() {
            let depth = self.get_mainline_depth(
                room_id,
                event_map.get(ev_id).cloned(),
                &mainline_map,
                store,
            );
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
        sort_event_ids.sort_by_key(|sort_id| order_map.get(sort_id).unwrap());

        sort_event_ids
    }

    fn get_mainline_depth(
        &mut self,
        room_id: &RoomId,
        mut event: Option<StateEvent>,
        mainline_map: &EventMap<usize>,
        store: &dyn StateStore,
    ) -> usize {
        while let Some(sort_ev) = event {
            if let Some(id) = sort_ev.event_id() {
                if let Some(depth) = mainline_map.get(id) {
                    return *depth;
                }
            }

            let auth_events = if let Some(id) = sort_ev.event_id() {
                store.auth_event_ids(room_id, id).unwrap()
            } else {
                vec![]
            };
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
            graph.insert(eid.clone(), vec![]);

            for aid in store.auth_event_ids(room_id, &eid).unwrap() {
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
