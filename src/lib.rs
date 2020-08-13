#![allow(clippy::or_fun_call)]

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

mod error;
pub mod event_auth;
pub mod room_version;
mod state_event;
mod state_store;

pub use error::{Error, Result};
pub use event_auth::{auth_check, auth_types_for_event};
pub use state_event::{Requester, StateEvent};
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
pub type StateMap<T> = BTreeMap<(EventType, Option<String>), T>;

/// A mapping of `EventId` to `T`, usually a `StateEvent`.
pub type EventMap<T> = BTreeMap<EventId, T>;

#[derive(Default)]
pub struct StateResolution;

impl StateResolution {
    /// Resolve sets of state events as they come in. Internally `StateResolution` builds a graph
    /// and an auth chain to allow for state conflict resolution.
    pub fn resolve(
        &self,
        room_id: &RoomId,
        room_version: &RoomVersionId,
        state_sets: &[StateMap<EventId>],
        event_map: Option<EventMap<StateEvent>>,
        store: &dyn StateStore,
        // TODO actual error handling (`thiserror`??)
    ) -> Result<ResolutionResult> {
        tracing::info!("State resolution starting");

        let mut event_map = if let Some(ev_map) = event_map {
            ev_map
        } else {
            EventMap::new()
        };
        // split non-conflicting and conflicting state
        let (clean, conflicting) = self.separate(&state_sets);

        tracing::info!("non conflicting {:?}", clean.len());

        if conflicting.is_empty() {
            tracing::info!("no conflicting state found");
            return Ok(ResolutionResult::Resolved(clean));
        }

        tracing::info!("{} conflicting events", conflicting.len());

        // the set of auth events that are not common across server forks
        let mut auth_diff = self.get_auth_chain_diff(room_id, &state_sets, store)?;

        tracing::debug!("auth diff size {}", auth_diff.len());

        // add the auth_diff to conflicting now we have a full set of conflicting events
        auth_diff.extend(conflicting.values().cloned().flatten());
        let mut all_conflicted = auth_diff
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        tracing::info!("full conflicted set is {} events", all_conflicted.len());

        // gather missing events for the event_map
        let events = store
            .get_events(
                room_id,
                &all_conflicted
                    .iter()
                    // we only want the events we don't know about yet
                    .filter(|id| !event_map.contains_key(id))
                    .cloned()
                    .collect::<Vec<_>>(),
            )
            .unwrap();

        // update event_map to include the fetched events
        event_map.extend(events.into_iter().map(|ev| (ev.event_id(), ev)));
        // at this point our event_map == store there should be no missing events

        tracing::debug!("event map size: {}", event_map.len());

        for event in event_map.values() {
            if event.room_id() != Some(room_id) {
                return Err(Error::TempString(format!(
                    "resolving event {} in room {}, when correct room is {}",
                    event.event_id(),
                    event.room_id().map(|id| id.as_str()).unwrap_or("`unknown`"),
                    room_id.as_str()
                )));
            }
        }

        // synapse says `full_set = {eid for eid in full_conflicted_set if eid in event_map}`
        //
        // don't honor events we cannot "verify"
        all_conflicted.retain(|id| event_map.contains_key(id));

        // get only the power events with a state_key: "" or ban/kick event (sender != state_key)
        let power_events = all_conflicted
            .iter()
            .filter(|id| is_power_event(id, &event_map))
            .cloned()
            .collect::<Vec<_>>();

        // sort the power events based on power_level/clock/event_id and outgoing/incoming edges
        let mut sorted_power_levels = self.reverse_topological_power_sort(
            room_id,
            &power_events,
            &mut event_map, // TODO use event_map
            store,
            &all_conflicted,
        );

        tracing::debug!(
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

        // we have resolved the power events so remove them, I'm sure there are other reasons to do so
        let events_to_resolve = all_conflicted
            .iter()
            .filter(|id| !deduped_power_ev.contains(id))
            .cloned()
            .collect::<Vec<_>>();

        tracing::debug!(
            "LEFT {:?}",
            events_to_resolve
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );

        let power_event = resolved.get(&(EventType::RoomPowerLevels, Some("".into())));

        tracing::debug!("PL {:?}", power_event);

        let sorted_left_events = self.mainline_sort(
            room_id,
            &events_to_resolve,
            power_event,
            &mut event_map,
            store,
        );

        tracing::debug!(
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
        &self,
        state_sets: &[StateMap<EventId>],
    ) -> (StateMap<EventId>, StateMap<Vec<EventId>>) {
        use itertools::Itertools;

        tracing::info!(
            "seperating {} sets of events into conflicted/unconflicted",
            state_sets.len()
        );

        let mut unconflicted_state = StateMap::new();
        let mut conflicted_state = StateMap::new();

        for key in state_sets
            .iter()
            .flat_map(|map| map.keys())
            .dedup()
            .collect::<Vec<_>>()
        {
            let mut event_ids = state_sets
                .iter()
                .map(|state_set| state_set.get(key))
                .dedup()
                .collect::<Vec<_>>();

            tracing::debug!(
                "SEP {:?}",
                event_ids
                    .iter()
                    .map(|i| i.map(ToString::to_string).unwrap_or("None".into()))
                    .collect::<Vec<_>>()
            );

            if event_ids.len() == 1 {
                if let Some(Some(id)) = event_ids.pop() {
                    unconflicted_state.insert(key.clone(), id.clone());
                } else {
                    panic!()
                }
            } else {
                conflicted_state.insert(
                    key.clone(),
                    event_ids.into_iter().flatten().cloned().collect::<Vec<_>>(),
                );
            }
        }

        (unconflicted_state, conflicted_state)
    }

    /// Returns a Vec of deduped EventIds that appear in some chains but no others.
    pub fn get_auth_chain_diff(
        &self,
        room_id: &RoomId,
        state_sets: &[StateMap<EventId>],
        store: &dyn StateStore,
    ) -> Result<Vec<EventId>> {
        use itertools::Itertools;

        tracing::debug!("calculating auth chain difference");

        store
            .auth_chain_diff(
                room_id,
                state_sets
                    .iter()
                    .map(|map| map.values().cloned().collect())
                    .dedup()
                    .collect::<Vec<_>>(),
            )
            .map_err(Error::TempString)
    }

    pub fn reverse_topological_power_sort(
        &self,
        room_id: &RoomId,
        events_to_sort: &[EventId],
        event_map: &mut EventMap<StateEvent>,
        store: &dyn StateStore,
        auth_diff: &[EventId],
    ) -> Vec<EventId> {
        tracing::debug!("reverse topological sort of power events");

        let mut graph = BTreeMap::new();
        for (idx, event_id) in events_to_sort.iter().enumerate() {
            self.add_event_and_auth_chain_to_graph(
                room_id, &mut graph, event_id, event_map, store, auth_diff,
            );

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
            tracing::info!("{} power level {}", event_id.to_string(), pl);

            event_to_pl.insert(event_id.clone(), pl);

            // We yield occasionally when we're working with large data sets to
            // ensure that we don't block the reactor loop for too long.
            if idx % _YIELD_AFTER_ITERATIONS == 0 {
                // yield clock.sleep(0)
            }
        }

        self.lexicographical_topological_sort(&graph, |event_id| {
            // tracing::debug!("{:?}", event_map.get(event_id).unwrap().origin_server_ts());
            let ev = event_map.get(event_id).unwrap();
            let pl = event_to_pl.get(event_id).unwrap();

            tracing::debug!("{:?}", (-*pl, *ev.origin_server_ts(), ev.event_id()));

            // count_0.sort_by(|(x, _), (y, _)| {
            //     x.power_level
            //         .cmp(&a.power_level)
            //         .then_with(|| x.origin_server.ts.cmp(&y.origin_server_ts))
            //         .then_with(|| x.event_id.cmp(&y.event_id))

            // This return value is the key used for sorting events,
            // events are then sorted by power level, time,
            // and lexically by event_id.
            (-*pl, *ev.origin_server_ts(), ev.event_id())
        })
    }

    /// Sorts the event graph based on number of outgoing/incoming edges, where
    /// `key_fn` is used as a tie breaker. The tie breaker happens based on
    /// power level, age, and event_id.
    pub fn lexicographical_topological_sort<F>(
        &self,
        graph: &BTreeMap<EventId, Vec<EventId>>,
        key_fn: F,
    ) -> Vec<EventId>
    where
        F: Fn(&EventId) -> (i64, SystemTime, EventId),
    {
        tracing::info!("starting lexicographical topological sort");
        // NOTE: an event that has no incoming edges happened most recently,
        // and an event that has no outgoing edges happened least recently.

        // NOTE: this is basically Kahn's algorithm except we look at nodes with no
        // outgoing edges, c.f.
        // https://en.wikipedia.org/wiki/Topological_sorting#Kahn's_algorithm

        // TODO make the BTreeSet conversion cleaner ??
        let mut outdegree_map: BTreeMap<EventId, BTreeSet<EventId>> = graph
            .iter()
            .map(|(k, v)| (k.clone(), v.iter().cloned().collect()))
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

        sorted
    }

    fn get_power_level_for_sender(
        &self,
        room_id: &RoomId,
        event_id: &EventId,
        event_map: &mut EventMap<StateEvent>,
        store: &dyn StateStore,
    ) -> i64 {
        tracing::info!("fetch event senders ({}) power level", event_id.to_string());
        let event = self._get_event(room_id, event_id, event_map, store);
        let mut pl = None;

        // TODO store.auth_event_ids returns "self" with the event ids is this ok
        // event.auth_event_ids does not include its own event id ?
        for aid in event.as_ref().unwrap().auth_events() {
            if let Some(aev) = self._get_event(room_id, &aid, event_map, store) {
                if aev.is_type_and_key(EventType::RoomPowerLevels, "") {
                    pl = Some(aev);
                    break;
                }
            }
        }

        if pl.is_none() {
            for aid in store.get_event(room_id, event_id).unwrap().auth_events() {
                if let Ok(aev) = store.get_event(room_id, &aid) {
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
                    tracing::debug!("found {} at power_level {}", ev.sender().to_string(), user);
                    return (*user).into();
                }
            }
            content.users_default.into()
        } else {
            0
        }
    }

    fn iterative_auth_check(
        &self,
        room_id: &RoomId,
        room_version: &RoomVersionId,
        power_events: &[EventId],
        unconflicted_state: &StateMap<EventId>,
        event_map: &mut EventMap<StateEvent>, // TODO use event_map over store ??
        store: &dyn StateStore,
    ) -> Result<StateMap<EventId>> {
        tracing::info!("starting iterative auth check");

        tracing::debug!(
            "{:?}",
            power_events
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );

        let mut resolved_state = unconflicted_state.clone();

        for (idx, event_id) in power_events.iter().enumerate() {
            let event = self
                ._get_event(room_id, event_id, event_map, store)
                .unwrap();

            let mut auth_events = BTreeMap::new();
            for aid in event.auth_events() {
                if let Some(ev) = self._get_event(room_id, &aid, event_map, store) {
                    // TODO what to do when no state_key is found ??
                    // TODO synapse check "rejected_reason", I'm guessing this is redacted_because for ruma ??
                    auth_events.insert((ev.kind(), ev.state_key()), ev);
                } else {
                    tracing::warn!("auth event id for {} is missing {}", aid, event_id);
                }
            }

            for key in event_auth::auth_types_for_event(&event) {
                if let Some(ev_id) = resolved_state.get(&key) {
                    if let Some(event) = self._get_event(room_id, ev_id, event_map, store) {
                        // TODO synapse checks `rejected_reason` is None here
                        auth_events.insert(key.clone(), event);
                    }
                }
            }

            tracing::debug!("event to check {:?}", event.event_id().to_string());

            if event_auth::auth_check(room_version, &event, auth_events, false)
                .ok_or("Auth check failed due to deserialization most likely".to_string())
                .map_err(Error::TempString)?
            {
                // add event to resolved state map
                resolved_state.insert((event.kind(), event.state_key()), event_id.clone());
            } else {
                // synapse passes here on AuthError. We do not add this event to resolved_state.
                tracing::warn!(
                    "event {} failed the authentication check",
                    event_id.to_string()
                );
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
    ///
    /// NOTE we rely on the `event_map` beign full at this point.
    /// TODO is this ok?
    fn mainline_sort(
        &self,
        room_id: &RoomId,
        to_sort: &[EventId],
        resolved_power_level: Option<&EventId>,
        event_map: &mut EventMap<StateEvent>,
        store: &dyn StateStore,
    ) -> Vec<EventId> {
        tracing::debug!("mainline sort of remaining events");

        // There are no EventId's to sort, bail.
        if to_sort.is_empty() {
            return vec![];
        }

        let mut mainline = vec![];
        let mut pl = resolved_power_level.cloned();
        let mut idx = 0;
        while let Some(p) = pl {
            mainline.push(p.clone());

            let event = self._get_event(room_id, &p, event_map, store).unwrap();
            let auth_events = event.auth_events();
            pl = None;
            for aid in auth_events {
                let ev = self._get_event(room_id, &aid, event_map, store).unwrap();
                if ev.is_type_and_key(EventType::RoomPowerLevels, "") {
                    pl = Some(aid.clone());
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
            let event = self._get_event(room_id, ev_id, event_map, store);
            let depth = self.get_mainline_depth(room_id, event, &mainline_map, event_map, store);
            order_map.insert(
                ev_id,
                (
                    depth,
                    event_map
                        .get(ev_id)
                        .map(|ev| ev.origin_server_ts())
                        .cloned(),
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

    // TODO make `event` not clone every loop
    fn get_mainline_depth(
        &self,
        room_id: &RoomId,
        mut event: Option<StateEvent>,
        mainline_map: &EventMap<usize>,
        event_map: &mut EventMap<StateEvent>,
        store: &dyn StateStore,
    ) -> usize {
        while let Some(sort_ev) = event {
            tracing::debug!("mainline event_id {}", sort_ev.event_id().to_string());
            let id = sort_ev.event_id();
            if let Some(depth) = mainline_map.get(&id) {
                return *depth;
            }

            let auth_events = sort_ev.auth_events();
            event = None;
            for aid in auth_events {
                let aev = self._get_event(room_id, &aid, event_map, store).unwrap();
                if aev.is_type_and_key(EventType::RoomPowerLevels, "") {
                    event = Some(aev.clone());
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
        event_map: &mut EventMap<StateEvent>,
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
            for aid in self
                ._get_event(room_id, &eid, event_map, store)
                .unwrap()
                .auth_events()
            {
                if auth_diff.contains(&aid) {
                    if !graph.contains_key(&aid) {
                        state.push(aid.clone());
                    }

                    // we just inserted this at the start of the while loop
                    graph.get_mut(&eid).unwrap().push(aid.clone());
                }
            }
        }
    }

    /// TODO update self if we go that route just as event_map will be updated
    fn _get_event(
        &self,
        room_id: &RoomId,
        ev_id: &EventId,
        event_map: &mut EventMap<StateEvent>,
        store: &dyn StateStore,
    ) -> Option<StateEvent> {
        // TODO can we cut down on the clones?
        if !event_map.contains_key(ev_id) {
            let event = store.get_event(room_id, ev_id).ok()?;
            event_map.insert(ev_id.clone(), event.clone());
            Some(event)
        } else {
            event_map.get(ev_id).cloned()
        }
    }
}

pub fn is_power_event(event_id: &EventId, event_map: &EventMap<StateEvent>) -> bool {
    match event_map.get(event_id) {
        Some(state) => state.is_power_event(),
        _ => false, // TODO this shouldn't eat errors?
    }
}
