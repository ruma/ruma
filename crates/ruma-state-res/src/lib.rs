use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, BinaryHeap},
    sync::Arc,
};

use maplit::btreeset;
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events::{
    room::{
        member::{MemberEventContent, MembershipState},
        power_levels::PowerLevelsEventContent,
    },
    EventType,
};
use ruma_identifiers::{EventId, RoomId, RoomVersionId};
use tracing::{debug, info, warn};

mod error;
pub mod event_auth;
pub mod room_version;
mod state_event;

pub use error::{Error, Result};
pub use event_auth::{auth_check, auth_types_for_event};
pub use room_version::RoomVersion;
pub use state_event::Event;

/// A mapping of event type and state_key to some value `T`, usually an `EventId`.
pub type StateMap<T> = BTreeMap<(EventType, String), T>;

/// A mapping of `EventId` to `T`, usually a `ServerPdu`.
pub type EventMap<T> = BTreeMap<EventId, T>;

#[derive(Default)]
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
    /// * `auth_events` - The full recursive set of `auth_events` for each event in the
    ///   `state_sets`.
    ///
    /// * `event_map` - The `EventMap` acts as a local cache of state, any event that is not found
    /// in the `event_map` will cause an unrecoverable `Error` in `resolve`.
    pub fn resolve<E: Event>(
        room_id: &RoomId,
        room_version: &RoomVersionId,
        state_sets: &[StateMap<EventId>],
        auth_events: Vec<Vec<EventId>>,
        event_map: &mut EventMap<Arc<E>>,
    ) -> Result<StateMap<EventId>> {
        info!("State resolution starting");

        // Split non-conflicting and conflicting state
        let (clean, conflicting) = StateResolution::separate(state_sets);

        info!("non conflicting {:?}", clean.len());

        if conflicting.is_empty() {
            info!("no conflicting state found");
            return Ok(clean);
        }

        info!("{} conflicting events", conflicting.len());

        // The set of auth events that are not common across server forks
        let mut auth_diff = StateResolution::get_auth_chain_diff(room_id, &auth_events)?;

        debug!("auth diff size {:?}", auth_diff);

        // Add the auth_diff to conflicting now we have a full set of conflicting events
        auth_diff.extend(conflicting.values().cloned().flatten());

        // `all_conflicted` contains unique items
        // synapse says `full_set = {eid for eid in full_conflicted_set if eid in event_map}`
        //
        // Don't honor events we cannot "verify"
        // TODO: BTreeSet::retain() when stable 1.53
        let all_conflicted =
            auth_diff.into_iter().filter(|id| event_map.contains_key(id)).collect::<BTreeSet<_>>();

        info!("full conflicted set is {} events", all_conflicted.len());

        // We used to check that all events are events from the correct room
        // this is now a check the caller of `resolve` must make.

        // Get only the control events with a state_key: "" or ban/kick event (sender != state_key)
        let control_events = all_conflicted
            .iter()
            .filter(|id| is_power_event_id(id, event_map))
            .cloned()
            .collect::<Vec<_>>();

        // Sort the control events based on power_level/clock/event_id and outgoing/incoming edges
        let sorted_control_levels = StateResolution::reverse_topological_power_sort(
            room_id,
            &control_events,
            event_map,
            &all_conflicted,
        );

        debug!("SRTD {:?}", sorted_control_levels);

        let room_version = RoomVersion::new(room_version)?;
        // Sequentially auth check each control event.
        let resolved_control = StateResolution::iterative_auth_check(
            room_id,
            &room_version,
            &sorted_control_levels,
            &clean,
            event_map,
        )?;

        debug!("AUTHED {:?}", resolved_control.iter().collect::<Vec<_>>());

        // At this point the control_events have been resolved we now have to
        // sort the remaining events using the mainline of the resolved power level.
        let deduped_power_ev = sorted_control_levels.into_iter().collect::<BTreeSet<_>>();

        // This removes the control events that passed auth and more importantly those that failed
        // auth
        let events_to_resolve = all_conflicted
            .iter()
            .filter(|id| !deduped_power_ev.contains(id))
            .cloned()
            .collect::<Vec<_>>();

        debug!("LEFT {:?}", events_to_resolve.iter().collect::<Vec<_>>());

        // This "epochs" power level event
        let power_event = resolved_control.get(&(EventType::RoomPowerLevels, "".into()));

        debug!("PL {:?}", power_event);

        let sorted_left_events =
            StateResolution::mainline_sort(room_id, &events_to_resolve, power_event, event_map);

        debug!("SORTED LEFT {:?}", sorted_left_events.iter().collect::<Vec<_>>());

        let mut resolved_state = StateResolution::iterative_auth_check(
            room_id,
            &room_version,
            &sorted_left_events,
            &resolved_control, // The control events are added to the final resolved state
            event_map,
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
    ) -> (StateMap<EventId>, StateMap<Vec<EventId>>) {
        use itertools::Itertools;

        info!("separating {} sets of events into conflicted/unconflicted", state_sets.len());

        let mut unconflicted_state = StateMap::new();
        let mut conflicted_state = StateMap::new();

        for key in state_sets.iter().flat_map(|map| map.keys()).unique() {
            let mut event_ids =
                state_sets.iter().map(|state_set| state_set.get(key)).unique().collect::<Vec<_>>();

            if event_ids.len() == 1 {
                let id = event_ids.remove(0).expect("unconflicting `EventId` is not None");
                unconflicted_state.insert(key.clone(), id.clone());
            } else {
                conflicted_state.insert(
                    key.clone(),
                    event_ids.into_iter().flatten().cloned().collect::<Vec<_>>(),
                );
            }
        }

        (unconflicted_state, conflicted_state)
    }

    /// Returns a Vec of deduped EventIds that appear in some chains but not others.
    pub fn get_auth_chain_diff(
        _room_id: &RoomId,
        auth_event_ids: &[Vec<EventId>],
    ) -> Result<BTreeSet<EventId>> {
        let mut chains = vec![];

        for ids in auth_event_ids {
            // TODO state store `auth_event_ids` returns self in the event ids list
            // when an event returns `auth_event_ids` self is not contained
            let chain = ids.iter().cloned().collect::<BTreeSet<_>>();
            chains.push(chain);
        }

        if let Some(chain) = chains.first().cloned() {
            let rest = chains.iter().skip(1).flatten().cloned().collect();
            let common = chain.intersection(&rest).collect::<Vec<_>>();

            Ok(chains.into_iter().flatten().filter(|id| !common.contains(&id)).collect())
        } else {
            Ok(btreeset![])
        }
    }

    /// Events are sorted from "earliest" to "latest". They are compared using
    /// the negative power level (reverse topological ordering), the
    /// origin server timestamp and incase of a tie the `EventId`s
    /// are compared lexicographically.
    ///
    /// The power level is negative because a higher power level is equated to an
    /// earlier (further back in time) origin server timestamp.
    pub fn reverse_topological_power_sort<E: Event>(
        room_id: &RoomId,
        events_to_sort: &[EventId],
        event_map: &mut EventMap<Arc<E>>,
        auth_diff: &BTreeSet<EventId>,
    ) -> Vec<EventId> {
        debug!("reverse topological sort of power events");

        let mut graph = BTreeMap::new();
        for event_id in events_to_sort.iter() {
            StateResolution::add_event_and_auth_chain_to_graph(
                room_id, &mut graph, event_id, event_map, auth_diff,
            );

            // TODO: if these functions are ever made async here
            // is a good place to yield every once in a while so other
            // tasks can make progress
        }

        // This is used in the `key_fn` passed to the lexico_topo_sort fn
        let mut event_to_pl = BTreeMap::new();
        for event_id in graph.keys() {
            let pl = StateResolution::get_power_level_for_sender(room_id, event_id, event_map);
            info!("{} power level {}", event_id, pl);

            event_to_pl.insert(event_id.clone(), pl);

            // TODO: if these functions are ever made async here
            // is a good place to yield every once in a while so other
            // tasks can make progress
        }

        StateResolution::lexicographical_topological_sort(&graph, |event_id| {
            let ev = event_map.get(event_id).unwrap();
            let pl = event_to_pl.get(event_id).unwrap();

            debug!("{:?}", (-*pl, ev.origin_server_ts(), &ev.event_id()));

            // This return value is the key used for sorting events,
            // events are then sorted by power level, time,
            // and lexically by event_id.
            (-*pl, ev.origin_server_ts(), ev.event_id().clone())
        })
    }

    /// Sorts the event graph based on number of outgoing/incoming edges, where
    /// `key_fn` is used as a tie breaker. The tie breaker happens based on
    /// power level, age, and event_id.
    pub fn lexicographical_topological_sort<F>(
        graph: &BTreeMap<EventId, BTreeSet<EventId>>,
        key_fn: F,
    ) -> Vec<EventId>
    where
        F: Fn(&EventId) -> (i64, MilliSecondsSinceUnixEpoch, EventId),
    {
        info!("starting lexicographical topological sort");
        // NOTE: an event that has no incoming edges happened most recently,
        // and an event that has no outgoing edges happened least recently.

        // NOTE: this is basically Kahn's algorithm except we look at nodes with no
        // outgoing edges, c.f.
        // https://en.wikipedia.org/wiki/Topological_sorting#Kahn's_algorithm

        // TODO make the BTreeSet conversion cleaner ??
        // outdegree_map is an event referring to the events before it, the
        // more outdegree's the more recent the event.
        let mut outdegree_map = graph.clone();

        // The number of events that depend on the given event (the eventId key)
        let mut reverse_graph = BTreeMap::new();

        // Vec of nodes that have zero out degree, least recent events.
        let mut zero_outdegree = vec![];

        for (node, edges) in graph.iter() {
            if edges.is_empty() {
                // The `Reverse` is because rusts `BinaryHeap` sorts largest -> smallest we need
                // smallest -> largest
                zero_outdegree.push(Reverse((key_fn(node), node)));
            }

            reverse_graph.entry(node).or_insert(btreeset![]);
            for edge in edges {
                reverse_graph.entry(edge).or_insert(btreeset![]).insert(node);
            }
        }

        let mut heap = BinaryHeap::from(zero_outdegree);

        // We remove the oldest node (most incoming edges) and check against all other
        let mut sorted = vec![];
        // Destructure the `Reverse` and take the smallest `node` each time
        while let Some(Reverse((_, node))) = heap.pop() {
            let node: &EventId = node;
            for parent in reverse_graph.get(node).unwrap() {
                // The number of outgoing edges this node has
                let out = outdegree_map.get_mut(parent).unwrap();

                // Only push on the heap once older events have been cleared
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

    /// Find the power level for the sender of `event_id` or return a default value of zero.
    fn get_power_level_for_sender<E: Event>(
        room_id: &RoomId,
        event_id: &EventId,
        event_map: &mut EventMap<Arc<E>>,
    ) -> i64 {
        info!("fetch event ({}) senders power level", event_id);

        let event = StateResolution::get_or_load_event(room_id, event_id, event_map);
        let mut pl = None;

        // TODO store.auth_event_ids returns "self" with the event ids is this ok
        // event.auth_event_ids does not include its own event id ?
        for aid in event.as_ref().map(|pdu| pdu.auth_events()).unwrap_or_default() {
            if let Ok(aev) = StateResolution::get_or_load_event(room_id, &aid, event_map) {
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
            if let Ok(ev) = event {
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
    pub fn iterative_auth_check<E: Event>(
        room_id: &RoomId,
        room_version: &RoomVersion,
        events_to_check: &[EventId],
        unconflicted_state: &StateMap<EventId>,
        event_map: &mut EventMap<Arc<E>>,
    ) -> Result<StateMap<EventId>> {
        info!("starting iterative auth check");

        debug!("performing auth checks on {:?}", events_to_check.iter().collect::<Vec<_>>());

        let mut resolved_state = unconflicted_state.clone();

        for event_id in events_to_check.iter() {
            let event = StateResolution::get_or_load_event(room_id, event_id, event_map)?;
            let state_key = event
                .state_key()
                .ok_or_else(|| Error::InvalidPdu("State event had no state key".to_owned()))?;

            let mut auth_events = BTreeMap::new();
            for aid in &event.auth_events() {
                if let Ok(ev) = StateResolution::get_or_load_event(room_id, aid, event_map) {
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
                    if let Ok(event) = StateResolution::get_or_load_event(room_id, ev_id, event_map)
                    {
                        // TODO synapse checks `rejected_reason` is None here
                        auth_events.insert(key.clone(), event);
                    }
                }
            }

            debug!("event to check {:?}", event.event_id());

            let most_recent_prev_event = event
                .prev_events()
                .iter()
                .filter_map(|id| StateResolution::get_or_load_event(room_id, id, event_map).ok())
                .next_back();

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
    pub fn mainline_sort<E: Event>(
        room_id: &RoomId,
        to_sort: &[EventId],
        resolved_power_level: Option<&EventId>,
        event_map: &mut EventMap<Arc<E>>,
    ) -> Vec<EventId> {
        debug!("mainline sort of events");

        // There are no EventId's to sort, bail.
        if to_sort.is_empty() {
            return vec![];
        }

        let mut mainline = vec![];
        let mut pl = resolved_power_level.cloned();
        while let Some(p) = pl {
            mainline.push(p.clone());

            let event = StateResolution::get_or_load_event(room_id, &p, event_map).unwrap();
            let auth_events = &event.auth_events();
            pl = None;
            for aid in auth_events {
                let ev = StateResolution::get_or_load_event(room_id, aid, event_map).unwrap();
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
            .collect::<BTreeMap<_, _>>();

        let mut order_map = BTreeMap::new();
        for ev_id in to_sort.iter() {
            if let Ok(event) = StateResolution::get_or_load_event(room_id, ev_id, event_map) {
                if let Ok(depth) = StateResolution::get_mainline_depth(
                    room_id,
                    Some(event),
                    &mainline_map,
                    event_map,
                ) {
                    order_map.insert(
                        ev_id,
                        (
                            depth,
                            event_map.get(ev_id).map(|ev| ev.origin_server_ts()),
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

        sort_event_ids
    }

    /// Get the mainline depth from the `mainline_map` or finds a power_level event
    /// that has an associated mainline depth.
    fn get_mainline_depth<E: Event>(
        room_id: &RoomId,
        mut event: Option<Arc<E>>,
        mainline_map: &EventMap<usize>,
        event_map: &mut EventMap<Arc<E>>,
    ) -> Result<usize> {
        while let Some(sort_ev) = event {
            debug!("mainline event_id {}", sort_ev.event_id());
            let id = &sort_ev.event_id();
            if let Some(depth) = mainline_map.get(id) {
                return Ok(*depth);
            }

            let auth_events = &sort_ev.auth_events();
            event = None;
            for aid in auth_events {
                let aev = StateResolution::get_or_load_event(room_id, aid, event_map)?;
                if is_type_and_key(&aev, EventType::RoomPowerLevels, "") {
                    event = Some(aev);
                    break;
                }
            }
        }
        // Did not find a power level event so we default to zero
        Ok(0)
    }

    fn add_event_and_auth_chain_to_graph<E: Event>(
        room_id: &RoomId,
        graph: &mut BTreeMap<EventId, BTreeSet<EventId>>,
        event_id: &EventId,
        event_map: &mut EventMap<Arc<E>>,
        auth_diff: &BTreeSet<EventId>,
    ) {
        let mut state = vec![event_id.clone()];
        while !state.is_empty() {
            // We just checked if it was empty so unwrap is fine
            let eid = state.pop().unwrap();
            graph.entry(eid.clone()).or_insert(btreeset![]);
            // Prefer the store to event as the store filters dedups the events
            for aid in &StateResolution::get_or_load_event(room_id, &eid, event_map)
                .map(|ev| ev.auth_events())
                .unwrap_or_default()
            {
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

    /// Uses the `event_map` to return the full PDU or fails.
    fn get_or_load_event<E: Event>(
        _room_id: &RoomId,
        ev_id: &EventId,
        event_map: &EventMap<Arc<E>>,
    ) -> Result<Arc<E>> {
        event_map.get(ev_id).map_or_else(
            || Err(Error::NotFound(format!("EventId: {:?} not found", ev_id))),
            |e| Ok(Arc::clone(e)),
        )
    }
}

pub fn is_power_event_id<E: Event>(event_id: &EventId, event_map: &EventMap<Arc<E>>) -> bool {
    match event_map.get(event_id) {
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
