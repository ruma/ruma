use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet},
    sync::Arc,
};

use itertools::Itertools;
use maplit::hashset;
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events::{
    room::{
        member::{MemberEventContent, MembershipState},
        power_levels::PowerLevelsEventContent,
    },
    EventType,
};
use ruma_identifiers::{EventId, RoomId, RoomVersionId};
use tracing::{debug, info, trace, warn};

mod error;
pub mod event_auth;
pub mod room_version;
mod state_event;

pub use error::{Error, Result};
pub use event_auth::{auth_check, auth_types_for_event};
pub use room_version::RoomVersion;
pub use state_event::Event;

/// A mapping of event type and state_key to some value `T`, usually an `EventId`.
pub type StateMap<T> = HashMap<(EventType, String), T>;

/// A mapping of `EventId` to `T`, usually a `ServerPdu`.
pub type EventMap<T> = HashMap<EventId, T>;

#[derive(Default)]
#[allow(clippy::exhaustive_structs)]
pub struct StateResolution;

impl StateResolution {
    /// Resolve sets of state events as they come in. Internally `StateResolution` builds a graph
    /// and an auth chain to allow for state conflict resolution.
    ///
    /// ## Arguments
    ///
    /// * `state_sets` - The incoming state to resolve. Each `StateMap` represents a possible fork
    /// in the state of a room.
    ///
    /// * `auth_chain_sets` - The full recursive set of `auth_events` for each event in the
    ///   `state_sets`.
    ///
    /// * `fetch_event` - Any event not found in the `event_map` will defer to this closure to find
    /// the event.
    ///
    /// ## Invariants
    ///
    /// The caller of `StateResolution::resolve` must ensure that all the events are from the same
    /// room. Although this function takes a `RoomId` it does not check that each event is part
    /// of the same room.
    pub fn resolve<E, F>(
        room_id: &RoomId,
        room_version: &RoomVersionId,
        state_sets: &[StateMap<EventId>],
        auth_chain_sets: Vec<HashSet<EventId>>,
        fetch_event: F,
    ) -> Result<StateMap<EventId>>
    where
        E: Event,
        F: Fn(&EventId) -> Option<Arc<E>>,
    {
        info!("State resolution starting");

        // Split non-conflicting and conflicting state
        let (clean, conflicting) = StateResolution::separate(state_sets);

        info!("non conflicting events: {}", clean.len());
        trace!("{:?}", clean);

        if conflicting.is_empty() {
            info!("no conflicting state found");
            return Ok(clean);
        }

        info!("conflicting events: {}", conflicting.len());
        debug!("{:?}", conflicting);

        let mut iter = conflicting.values();
        let mut conflicting_state_sets = iter
            .next()
            .expect("we made sure conflicting is not empty")
            .iter()
            .map(|o| if let Some(e) = o { hashset![e.clone()] } else { HashSet::new() })
            .collect::<Vec<_>>();

        for events in iter {
            for i in 0..events.len() {
                // This is okay because all vecs have the same length = number of states
                if let Some(e) = &events[i] {
                    conflicting_state_sets[i].insert(e.clone());
                }
            }
        }

        // The set of auth events that are not common across server forks
        let mut auth_diff = StateResolution::get_auth_chain_diff(room_id, auth_chain_sets)?;

        // Add the auth_diff to conflicting now we have a full set of conflicting events
        auth_diff.extend(conflicting.values().cloned().flatten().flatten());

        debug!("auth diff: {}", auth_diff.len());
        trace!("{:?}", auth_diff);

        // `all_conflicted` contains unique items
        // synapse says `full_set = {eid for eid in full_conflicted_set if eid in event_map}`
        //
        // Don't honor events we cannot "verify"
        // TODO: BTreeSet::retain() when stable 1.53
        let all_conflicted =
            auth_diff.into_iter().filter(|id| fetch_event(id).is_some()).collect::<HashSet<_>>();

        info!("full conflicted set: {}", all_conflicted.len());
        debug!("{:?}", all_conflicted);

        // We used to check that all events are events from the correct room
        // this is now a check the caller of `resolve` must make.

        // Get only the control events with a state_key: "" or ban/kick event (sender != state_key)
        let control_events = all_conflicted
            .iter()
            .filter(|id| is_power_event_id(id, &fetch_event))
            .cloned()
            .collect::<Vec<_>>();

        // Sort the control events based on power_level/clock/event_id and outgoing/incoming edges
        let sorted_control_levels = StateResolution::reverse_topological_power_sort(
            &control_events,
            &all_conflicted,
            &fetch_event,
        )?;

        debug!("sorted control events: {}", sorted_control_levels.len());
        trace!("{:?}", sorted_control_levels);

        let room_version = RoomVersion::new(room_version)?;
        // Sequentially auth check each control event.
        let resolved_control = StateResolution::iterative_auth_check(
            &room_version,
            &sorted_control_levels,
            &clean,
            &fetch_event,
        )?;

        debug!("resolved control events: {}", resolved_control.len());
        trace!("{:?}", resolved_control);

        // At this point the control_events have been resolved we now have to
        // sort the remaining events using the mainline of the resolved power level.
        let deduped_power_ev = sorted_control_levels.into_iter().collect::<HashSet<_>>();

        // This removes the control events that passed auth and more importantly those that failed
        // auth
        let events_to_resolve = all_conflicted
            .iter()
            .filter(|id| !deduped_power_ev.contains(id))
            .cloned()
            .collect::<Vec<_>>();

        debug!("events left to resolve: {}", events_to_resolve.len());
        trace!("{:?}", events_to_resolve);

        // This "epochs" power level event
        let power_event = resolved_control.get(&(EventType::RoomPowerLevels, "".into()));

        debug!("power event: {:?}", power_event);

        let sorted_left_events =
            StateResolution::mainline_sort(&events_to_resolve, power_event, &fetch_event)?;

        trace!("events left, sorted: {:?}", sorted_left_events.iter().collect::<Vec<_>>());

        let mut resolved_state = StateResolution::iterative_auth_check(
            &room_version,
            &sorted_left_events,
            &resolved_control, // The control events are added to the final resolved state
            &fetch_event,
        )?;

        // Add unconflicted state to the resolved state
        // We priorities the unconflicting state
        resolved_state.extend(clean);
        Ok(resolved_state)
    }

    /// Split the events that have no conflicts from those that are conflicting.
    /// The return tuple looks like `(unconflicted, conflicted)`.
    ///
    /// State is determined to be conflicting if for the given key (EventType, StateKey) there
    /// is not exactly one eventId. This includes missing events, if one state_set includes an event
    /// that none of the other have this is a conflicting event.
    pub fn separate(
        state_sets: &[StateMap<EventId>],
    ) -> (StateMap<EventId>, StateMap<Vec<Option<EventId>>>) {
        info!("separating {} sets of events into conflicted/unconflicted", state_sets.len());

        let mut unconflicted_state = StateMap::new();
        let mut conflicted_state = StateMap::new();

        for key in state_sets.iter().flat_map(|map| map.keys()).unique() {
            let mut event_ids =
                state_sets.iter().map(|state_set| state_set.get(key)).collect::<Vec<_>>();

            if event_ids.iter().all_equal() {
                let id = event_ids.remove(0).expect("unconflicting `EventId` is not None");
                unconflicted_state.insert(key.clone(), id.clone());
            } else {
                conflicted_state
                    .insert(key.clone(), event_ids.into_iter().map(|o| o.cloned()).collect());
            }
        }

        (unconflicted_state, conflicted_state)
    }

    /// Returns a Vec of deduped EventIds that appear in some chains but not others.
    pub fn get_auth_chain_diff(
        _room_id: &RoomId,
        auth_chain_sets: Vec<HashSet<EventId>>,
    ) -> Result<HashSet<EventId>> {
        if let Some(first) = auth_chain_sets.first().cloned() {
            let common = auth_chain_sets
                .iter()
                .skip(1)
                .fold(first, |a, b| a.intersection(b).cloned().collect::<HashSet<EventId>>());

            Ok(auth_chain_sets.into_iter().flatten().filter(|id| !common.contains(id)).collect())
        } else {
            Ok(hashset![])
        }
    }

    /// Events are sorted from "earliest" to "latest". They are compared using
    /// the negative power level (reverse topological ordering), the
    /// origin server timestamp and incase of a tie the `EventId`s
    /// are compared lexicographically.
    ///
    /// The power level is negative because a higher power level is equated to an
    /// earlier (further back in time) origin server timestamp.
    pub fn reverse_topological_power_sort<E, F>(
        events_to_sort: &[EventId],
        auth_diff: &HashSet<EventId>,
        fetch_event: F,
    ) -> Result<Vec<EventId>>
    where
        E: Event,
        F: Fn(&EventId) -> Option<Arc<E>>,
    {
        debug!("reverse topological sort of power events");

        let mut graph = HashMap::new();
        for event_id in events_to_sort.iter() {
            StateResolution::add_event_and_auth_chain_to_graph(
                &mut graph,
                event_id,
                auth_diff,
                &fetch_event,
            );

            // TODO: if these functions are ever made async here
            // is a good place to yield every once in a while so other
            // tasks can make progress
        }

        // This is used in the `key_fn` passed to the lexico_topo_sort fn
        let mut event_to_pl = HashMap::new();
        for event_id in graph.keys() {
            let pl = StateResolution::get_power_level_for_sender(event_id, &fetch_event);
            info!("{} power level {}", event_id, pl);

            event_to_pl.insert(event_id.clone(), pl);

            // TODO: if these functions are ever made async here
            // is a good place to yield every once in a while so other
            // tasks can make progress
        }

        StateResolution::lexicographical_topological_sort(&graph, |event_id| {
            let ev = fetch_event(event_id).ok_or_else(|| Error::NotFound("".into()))?;
            let pl = event_to_pl.get(event_id).ok_or_else(|| Error::NotFound("".into()))?;

            debug!("{:?}", (-*pl, ev.origin_server_ts(), &ev.event_id()));

            // This return value is the key used for sorting events,
            // events are then sorted by power level, time,
            // and lexically by event_id.
            Ok((-*pl, ev.origin_server_ts(), ev.event_id().clone()))
        })
    }

    /// Sorts the event graph based on number of outgoing/incoming edges, where
    /// `key_fn` is used as a tie breaker. The tie breaker happens based on
    /// power level, age, and event_id.
    pub fn lexicographical_topological_sort<F>(
        graph: &HashMap<EventId, HashSet<EventId>>,
        key_fn: F,
    ) -> Result<Vec<EventId>>
    where
        F: Fn(&EventId) -> Result<(i64, MilliSecondsSinceUnixEpoch, EventId)>,
    {
        info!("starting lexicographical topological sort");
        // NOTE: an event that has no incoming edges happened most recently,
        // and an event that has no outgoing edges happened least recently.

        // NOTE: this is basically Kahn's algorithm except we look at nodes with no
        // outgoing edges, c.f.
        // https://en.wikipedia.org/wiki/Topological_sorting#Kahn's_algorithm

        // outdegree_map is an event referring to the events before it, the
        // more outdegree's the more recent the event.
        let mut outdegree_map = graph.clone();

        // The number of events that depend on the given event (the EventId key)
        // How many events reference this event in the DAG as a parent
        let mut reverse_graph = HashMap::new();

        // Vec of nodes that have zero out degree, least recent events.
        let mut zero_outdegree = vec![];

        for (node, edges) in graph.iter() {
            if edges.is_empty() {
                // The `Reverse` is because rusts `BinaryHeap` sorts largest -> smallest we need
                // smallest -> largest
                zero_outdegree.push(Reverse((key_fn(node)?, node)));
            }

            reverse_graph.entry(node).or_insert(hashset![]);
            for edge in edges {
                reverse_graph.entry(edge).or_insert(hashset![]).insert(node);
            }
        }

        let mut heap = BinaryHeap::from(zero_outdegree);

        // We remove the oldest node (most incoming edges) and check against all other
        let mut sorted = vec![];
        // Destructure the `Reverse` and take the smallest `node` each time
        while let Some(Reverse((_, node))) = heap.pop() {
            let node: &EventId = node;
            for parent in reverse_graph.get(node).expect("EventId in heap is also in reverse_graph")
            {
                // The number of outgoing edges this node has
                let out = outdegree_map
                    .get_mut(parent)
                    .expect("outdegree_map knows of all referenced EventIds");

                // Only push on the heap once older events have been cleared
                out.remove(node);
                if out.is_empty() {
                    heap.push(Reverse((key_fn(parent)?, parent)));
                }
            }

            // synapse yields we push then return the vec
            sorted.push(node.clone());
        }

        Ok(sorted)
    }

    /// Find the power level for the sender of `event_id` or return a default value of zero.
    fn get_power_level_for_sender<E, F>(event_id: &EventId, fetch_event: F) -> i64
    where
        E: Event,
        F: Fn(&EventId) -> Option<Arc<E>>,
    {
        info!("fetch event ({}) senders power level", event_id);

        let event = fetch_event(event_id);
        let mut pl = None;

        // TODO store.auth_event_ids returns "self" with the event ids is this ok
        // event.auth_event_ids does not include its own event id ?
        for aid in event.as_ref().map(|pdu| pdu.auth_events()).unwrap_or_default() {
            if let Some(aev) = fetch_event(&aid) {
                if is_type_and_key(&aev, EventType::RoomPowerLevels, "") {
                    pl = Some(aev);
                    break;
                }
            }
        }

        if pl.is_none() {
            return 0;
        }

        if let Some(content) =
            pl.and_then(|pl| serde_json::from_value::<PowerLevelsEventContent>(pl.content()).ok())
        {
            if let Some(ev) = event {
                if let Some(user) = content.users.get(ev.sender()) {
                    debug!("found {} at power_level {}", ev.sender(), user);
                    return (*user).into();
                }
            }
            content.users_default.into()
        } else {
            0
        }
    }

    /// Check the that each event is authenticated based on the events before it.
    ///
    /// ## Returns
    ///
    /// The `unconflicted_state` combined with the newly auth'ed events. So any event that
    /// fails the `event_auth::auth_check` will be excluded from the returned `StateMap<EventId>`.
    ///
    /// For each `events_to_check` event we gather the events needed to auth it from the
    /// `event_map` or `store` and verify each event using the `event_auth::auth_check`
    /// function.
    pub fn iterative_auth_check<E, F>(
        room_version: &RoomVersion,
        events_to_check: &[EventId],
        unconflicted_state: &StateMap<EventId>,
        fetch_event: F,
    ) -> Result<StateMap<EventId>>
    where
        E: Event,
        F: Fn(&EventId) -> Option<Arc<E>>,
    {
        info!("starting iterative auth check");

        debug!("performing auth checks on {:?}", events_to_check.iter().collect::<Vec<_>>());

        let mut resolved_state = unconflicted_state.clone();

        for event_id in events_to_check.iter() {
            let event = fetch_event(event_id)
                .ok_or_else(|| Error::NotFound(format!("Failed to find {}", event_id)))?;
            let state_key = event
                .state_key()
                .ok_or_else(|| Error::InvalidPdu("State event had no state key".to_owned()))?;

            let mut auth_events = HashMap::new();
            for aid in &event.auth_events() {
                if let Some(ev) = fetch_event(aid) {
                    // TODO synapse check "rejected_reason", I'm guessing this is redacted_because
                    // in ruma ??
                    auth_events.insert(
                        (
                            ev.kind(),
                            ev.state_key().ok_or_else(|| {
                                Error::InvalidPdu("State event had no state key".to_owned())
                            })?,
                        ),
                        ev,
                    );
                } else {
                    warn!("auth event id for {} is missing {}", aid, event_id);
                }
            }

            for key in auth_types_for_event(
                &event.kind(),
                event.sender(),
                Some(state_key.clone()),
                event.content(),
            ) {
                if let Some(ev_id) = resolved_state.get(&key) {
                    if let Some(event) = fetch_event(ev_id) {
                        // TODO synapse checks `rejected_reason` is None here
                        auth_events.insert(key.clone(), event);
                    }
                }
            }

            debug!("event to check {:?}", event.event_id());

            let most_recent_prev_event =
                event.prev_events().iter().filter_map(|id| fetch_event(id)).next_back();

            // The key for this is (eventType + a state_key of the signed token not sender) so
            // search for it
            let current_third_party = auth_events.iter().find_map(|(_, pdu)| {
                (pdu.kind() == EventType::RoomThirdPartyInvite).then(|| {
                    // TODO no clone, auth_events is borrowed while moved
                    pdu.clone()
                })
            });

            if auth_check(
                room_version,
                &event,
                most_recent_prev_event,
                &auth_events,
                current_third_party,
            )? {
                // add event to resolved state map
                resolved_state.insert((event.kind(), state_key), event_id.clone());
            } else {
                // synapse passes here on AuthError. We do not add this event to resolved_state.
                warn!("event {} failed the authentication check", event_id);
            }

            // TODO: if these functions are ever made async here
            // is a good place to yield every once in a while so other
            // tasks can make progress
        }
        Ok(resolved_state)
    }

    /// Returns the sorted `to_sort` list of `EventId`s based on a mainline sort using
    /// the depth of `resolved_power_level`, the server timestamp, and the eventId.
    ///
    /// The depth of the given event is calculated based on the depth of it's closest "parent"
    /// power_level event. If there have been two power events the after the most recent are
    /// depth 0, the events before (with the first power level as a parent) will be marked
    /// as depth 1. depth 1 is "older" than depth 0.
    pub fn mainline_sort<E, F>(
        to_sort: &[EventId],
        resolved_power_level: Option<&EventId>,
        fetch_event: F,
    ) -> Result<Vec<EventId>>
    where
        E: Event,
        F: Fn(&EventId) -> Option<Arc<E>>,
    {
        debug!("mainline sort of events");

        // There are no EventId's to sort, bail.
        if to_sort.is_empty() {
            return Ok(vec![]);
        }

        let mut mainline = vec![];
        let mut pl = resolved_power_level.cloned();
        while let Some(p) = pl {
            mainline.push(p.clone());

            let event =
                fetch_event(&p).ok_or_else(|| Error::NotFound(format!("Failed to find {}", p)))?;
            let auth_events = &event.auth_events();
            pl = None;
            for aid in auth_events {
                let ev = fetch_event(aid)
                    .ok_or_else(|| Error::NotFound(format!("Failed to find {}", aid)))?;
                if is_type_and_key(&ev, EventType::RoomPowerLevels, "") {
                    pl = Some(aid.clone());
                    break;
                }
            }
            // TODO: if these functions are ever made async here
            // is a good place to yield every once in a while so other
            // tasks can make progress
        }

        let mainline_map = mainline
            .iter()
            .rev()
            .enumerate()
            .map(|(idx, eid)| ((*eid).clone(), idx))
            .collect::<HashMap<_, _>>();

        let mut order_map = HashMap::new();
        for ev_id in to_sort.iter() {
            if let Some(event) = fetch_event(ev_id) {
                if let Ok(depth) =
                    StateResolution::get_mainline_depth(Some(event), &mainline_map, &fetch_event)
                {
                    order_map.insert(
                        ev_id,
                        (
                            depth,
                            fetch_event(ev_id).map(|ev| ev.origin_server_ts()),
                            ev_id, // TODO should this be a &str to sort lexically??
                        ),
                    );
                }
            }

            // TODO: if these functions are ever made async here
            // is a good place to yield every once in a while so other
            // tasks can make progress
        }

        // Sort the event_ids by their depth, timestamp and EventId
        // unwrap is OK order map and sort_event_ids are from to_sort (the same Vec)
        let mut sort_event_ids = order_map.keys().map(|&k| k.clone()).collect::<Vec<_>>();
        sort_event_ids.sort_by_key(|sort_id| order_map.get(sort_id).unwrap());

        Ok(sort_event_ids)
    }

    /// Get the mainline depth from the `mainline_map` or finds a power_level event
    /// that has an associated mainline depth.
    fn get_mainline_depth<E, F>(
        mut event: Option<Arc<E>>,
        mainline_map: &EventMap<usize>,
        fetch_event: F,
    ) -> Result<usize>
    where
        E: Event,
        F: Fn(&EventId) -> Option<Arc<E>>,
    {
        while let Some(sort_ev) = event {
            debug!("mainline event_id {}", sort_ev.event_id());
            let id = &sort_ev.event_id();
            if let Some(depth) = mainline_map.get(id) {
                return Ok(*depth);
            }

            let auth_events = &sort_ev.auth_events();
            event = None;
            for aid in auth_events {
                let aev = fetch_event(aid)
                    .ok_or_else(|| Error::NotFound(format!("Failed to find {}", aid)))?;
                if is_type_and_key(&aev, EventType::RoomPowerLevels, "") {
                    event = Some(aev);
                    break;
                }
            }
        }
        // Did not find a power level event so we default to zero
        Ok(0)
    }

    fn add_event_and_auth_chain_to_graph<E, F>(
        graph: &mut HashMap<EventId, HashSet<EventId>>,
        event_id: &EventId,
        auth_diff: &HashSet<EventId>,
        fetch_event: F,
    ) where
        E: Event,
        F: Fn(&EventId) -> Option<Arc<E>>,
    {
        let mut state = vec![event_id.clone()];
        while !state.is_empty() {
            // We just checked if it was empty so unwrap is fine
            let eid = state.pop().unwrap();
            graph.entry(eid.clone()).or_insert(hashset![]);
            // Prefer the store to event as the store filters dedups the events
            for aid in &fetch_event(&eid).map(|ev| ev.auth_events()).unwrap_or_default() {
                if auth_diff.contains(aid) {
                    if !graph.contains_key(aid) {
                        state.push(aid.clone());
                    }

                    // We just inserted this at the start of the while loop
                    graph.get_mut(&eid).unwrap().insert(aid.clone());
                }
            }
        }
    }
}

pub fn is_power_event_id<E, F>(event_id: &EventId, fetch: F) -> bool
where
    E: Event,
    F: Fn(&EventId) -> Option<Arc<E>>,
{
    match fetch(event_id).as_ref() {
        Some(state) => is_power_event(state),
        _ => false,
    }
}

pub fn is_type_and_key<E: Event>(ev: &Arc<E>, ev_type: EventType, state_key: &str) -> bool {
    ev.kind() == ev_type && ev.state_key().as_deref() == Some(state_key)
}

pub fn is_power_event<E: Event>(event: &Arc<E>) -> bool {
    match event.kind() {
        EventType::RoomPowerLevels | EventType::RoomJoinRules | EventType::RoomCreate => {
            event.state_key() == Some("".into())
        }
        EventType::RoomMember => {
            if let Ok(content) = serde_json::from_value::<MemberEventContent>(event.content()) {
                if [MembershipState::Leave, MembershipState::Ban].contains(&content.membership) {
                    return Some(event.sender().as_str()) != event.state_key().as_deref();
                }
            }

            false
        }
        _ => false,
    }
}
