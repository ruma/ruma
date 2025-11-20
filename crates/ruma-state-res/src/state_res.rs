use std::{
    borrow::Borrow,
    cmp::{Ordering, Reverse},
    collections::{BinaryHeap, HashMap, HashSet},
    hash::Hash,
    sync::OnceLock,
};

use ruma_common::{
    EventId, MilliSecondsSinceUnixEpoch, OwnedUserId,
    room_version_rules::{AuthorizationRules, StateResolutionV2Rules},
};
use ruma_events::{
    StateEventType, TimelineEventType,
    room::{member::MembershipState, power_levels::UserPowerLevel},
};
use tracing::{debug, info, instrument, trace, warn};

#[cfg(test)]
mod tests;

use crate::{
    Error, Event, Result, auth_types_for_event, check_state_dependent_auth_rules,
    events::{
        RoomCreateEvent, RoomMemberEvent, RoomPowerLevelsEvent, RoomPowerLevelsIntField,
        power_levels::RoomPowerLevelsEventOptionExt,
    },
    utils::RoomIdExt,
};

/// A mapping of event type and state_key to some value `T`, usually an `EventId`.
///
/// This is the representation of what the Matrix specification calls a "room state" or a "state
/// map" during [state resolution].
///
/// [state resolution]: https://spec.matrix.org/latest/rooms/v2/#state-resolution
pub type StateMap<T> = HashMap<(StateEventType, String), T>;

/// Apply the [state resolution] algorithm introduced in room version 2 to resolve the state of a
/// room.
///
/// ## Arguments
///
/// * `auth_rules` - The authorization rules to apply for the version of the current room.
///
/// * `state_res_rules` - The state resolution rules to apply for the version of the current room.
///
/// * `state_maps` - The incoming states to resolve. Each `StateMap` represents a possible fork in
///   the state of a room.
///
/// * `auth_chains` - The list of full recursive sets of `auth_events` for each event in the
///   `state_maps`.
///
/// * `fetch_event` - Function to fetch an event in the room given its event ID.
///
/// * `fetch_conflicted_state_subgraph` - Function to fetch the conflicted state subgraph for the
///   given conflicted state set, for state resolution rules that use it. If it is called and
///   returns `None`, this function will return an error.
///
/// ## Invariants
///
/// The caller of `resolve` must ensure that all the events are from the same room.
///
/// ## Returns
///
/// The resolved room state.
///
/// [state resolution]: https://spec.matrix.org/latest/rooms/v2/#state-resolution
#[instrument(skip_all)]
pub fn resolve<'a, E, MapsIter>(
    auth_rules: &AuthorizationRules,
    state_res_rules: &StateResolutionV2Rules,
    state_maps: impl IntoIterator<IntoIter = MapsIter>,
    auth_chains: Vec<HashSet<E::Id>>,
    fetch_event: impl Fn(&EventId) -> Option<E>,
    fetch_conflicted_state_subgraph: impl Fn(&StateMap<Vec<E::Id>>) -> Option<HashSet<E::Id>>,
) -> Result<StateMap<E::Id>>
where
    E: Event + Clone,
    E::Id: 'a,
    MapsIter: Iterator<Item = &'a StateMap<E::Id>> + Clone,
{
    info!("state resolution starting");

    // Split the unconflicted state map and the conflicted state set.
    let (unconflicted_state_map, conflicted_state_set) =
        split_conflicted_state_set(state_maps.into_iter());

    info!(count = unconflicted_state_map.len(), "unconflicted events");
    trace!(map = ?unconflicted_state_map, "unconflicted events");

    if conflicted_state_set.is_empty() {
        info!("no conflicted state found");
        return Ok(unconflicted_state_map);
    }

    info!(count = conflicted_state_set.len(), "conflicted events");
    trace!(map = ?conflicted_state_set, "conflicted events");

    // Since v12, fetch the conflicted state subgraph.
    let conflicted_state_subgraph = if state_res_rules.consider_conflicted_state_subgraph {
        let conflicted_state_subgraph = fetch_conflicted_state_subgraph(&conflicted_state_set)
            .ok_or(Error::FetchConflictedStateSubgraphFailed)?;

        info!(count = conflicted_state_subgraph.len(), "events in conflicted state subgraph");
        trace!(set = ?conflicted_state_subgraph, "conflicted state subgraph");

        conflicted_state_subgraph
    } else {
        HashSet::new()
    };

    // The full conflicted set is the union of the conflicted state set and the auth difference,
    // and since v12, the conflicted state subgraph.
    let full_conflicted_set: HashSet<_> = auth_difference(auth_chains)
        .chain(conflicted_state_set.into_values().flatten())
        .chain(conflicted_state_subgraph)
        // Don't honor events we cannot "verify"
        .filter(|id| fetch_event(id.borrow()).is_some())
        .collect();

    info!(count = full_conflicted_set.len(), "full conflicted set");
    trace!(set = ?full_conflicted_set, "full conflicted set");

    // 1. Select the set X of all power events that appear in the full conflicted set. For each such
    //    power event P, enlarge X by adding the events in the auth chain of P which also belong to
    //    the full conflicted set. Sort X into a list using the reverse topological power ordering.
    let conflicted_power_events = full_conflicted_set
        .iter()
        .filter(|&id| is_power_event_id(id.borrow(), &fetch_event))
        .cloned()
        .collect::<Vec<_>>();

    let sorted_power_events =
        sort_power_events(conflicted_power_events, &full_conflicted_set, auth_rules, &fetch_event)?;

    debug!(count = sorted_power_events.len(), "power events");
    trace!(list = ?sorted_power_events, "sorted power events");

    // 2. Apply the iterative auth checks algorithm, starting from the unconflicted state map, to
    //    the list of events from the previous step to get a partially resolved state.

    // Since v12, begin the first phase of iterative auth checks with an empty state map.
    let initial_state_map = if state_res_rules.begin_iterative_auth_checks_with_empty_state_map {
        HashMap::new()
    } else {
        unconflicted_state_map.clone()
    };

    let partially_resolved_state =
        iterative_auth_checks(auth_rules, &sorted_power_events, initial_state_map, &fetch_event)?;

    debug!(count = partially_resolved_state.len(), "resolved power events");
    trace!(map = ?partially_resolved_state, "resolved power events");

    // 3. Take all remaining events that weren’t picked in step 1 and order them by the mainline
    //    ordering based on the power level in the partially resolved state obtained in step 2.
    let sorted_power_events_set = sorted_power_events.into_iter().collect::<HashSet<_>>();
    let remaining_events = full_conflicted_set
        .iter()
        .filter(|&id| !sorted_power_events_set.contains(id.borrow()))
        .cloned()
        .collect::<Vec<_>>();

    debug!(count = remaining_events.len(), "events left to resolve");
    trace!(list = ?remaining_events, "events left to resolve");

    // This "epochs" power level event
    let power_event = partially_resolved_state.get(&(StateEventType::RoomPowerLevels, "".into()));

    debug!(event_id = ?power_event, "power event");

    let sorted_remaining_events =
        mainline_sort(&remaining_events, power_event.cloned(), &fetch_event)?;

    trace!(list = ?sorted_remaining_events, "events left, sorted");

    // 4. Apply the iterative auth checks algorithm on the partial resolved state and the list of
    //    events from the previous step.
    let mut resolved_state = iterative_auth_checks(
        auth_rules,
        &sorted_remaining_events,
        partially_resolved_state,
        &fetch_event,
    )?;

    // 5. Update the result by replacing any event with the event with the same key from the
    //    unconflicted state map, if such an event exists, to get the final resolved state.
    resolved_state.extend(unconflicted_state_map);

    info!("state resolution finished");

    Ok(resolved_state)
}

/// Split the unconflicted state map and the conflicted state set.
///
/// Definition in the specification:
///
/// > If a given key _K_ is present in every _Si_ with the same value _V_ in each state map, then
/// > the pair (_K_, _V_) belongs to the unconflicted state map. Otherwise, _V_ belongs to the
/// > conflicted state set.
///
/// It means that, for a given (event type, state key) tuple, if all state maps have the same event
/// ID, it lands in the unconflicted state map, otherwise the event IDs land in the conflicted state
/// set.
///
/// ## Arguments
///
/// * `state_maps` - The incoming states to resolve. Each `StateMap` represents a possible fork in
///   the state of a room.
///
/// ## Returns
///
/// Returns an `(unconflicted_state_map, conflicted_state_set)` tuple.
fn split_conflicted_state_set<'a, Id>(
    state_maps: impl Iterator<Item = &'a StateMap<Id>>,
) -> (StateMap<Id>, StateMap<Vec<Id>>)
where
    Id: Clone + Eq + Hash + 'a,
{
    let mut state_set_count = 0_usize;
    let mut occurrences = HashMap::<_, HashMap<_, _>>::new();

    let state_maps = state_maps.inspect(|_| state_set_count += 1);
    for (k, v) in state_maps.flatten() {
        occurrences.entry(k).or_default().entry(v).and_modify(|x| *x += 1).or_insert(1);
    }

    let mut unconflicted_state_map = StateMap::new();
    let mut conflicted_state_set = StateMap::new();

    for (k, v) in occurrences {
        for (id, occurrence_count) in v {
            if occurrence_count == state_set_count {
                unconflicted_state_map.insert((k.0.clone(), k.1.clone()), id.clone());
            } else {
                conflicted_state_set
                    .entry((k.0.clone(), k.1.clone()))
                    .and_modify(|x: &mut Vec<_>| x.push(id.clone()))
                    .or_insert(vec![id.clone()]);
            }
        }
    }

    (unconflicted_state_map, conflicted_state_set)
}

/// Get the auth difference for the given auth chains.
///
/// Definition in the specification:
///
/// > The auth difference is calculated by first calculating the full auth chain for each state
/// > _Si_, that is the union of the auth chains for each event in _Si_, and then taking every event
/// > that doesn’t appear in every auth chain. If _Ci_ is the full auth chain of _Si_, then the auth
/// > difference is ∪_Ci_ − ∩_Ci_.
///
/// ## Arguments
///
/// * `auth_chains` - The list of full recursive sets of `auth_events`.
///
/// ## Returns
///
/// Returns an iterator over all the event IDs that are not present in all the auth chains.
fn auth_difference<Id>(auth_chains: Vec<HashSet<Id>>) -> impl Iterator<Item = Id>
where
    Id: Eq + Hash,
{
    let num_sets = auth_chains.len();

    let mut id_counts: HashMap<Id, usize> = HashMap::new();
    for id in auth_chains.into_iter().flatten() {
        *id_counts.entry(id).or_default() += 1;
    }

    id_counts.into_iter().filter_map(move |(id, count)| (count < num_sets).then_some(id))
}

/// Enlarge the given list of conflicted power events by adding the events in their auth chain that
/// are in the full conflicted set, and sort it using reverse topological power ordering.
///
/// ## Arguments
///
/// * `conflicted_power_events` - The list of power events in the full conflicted set.
///
/// * `full_conflicted_set` - The full conflicted set.
///
/// * `rules` - The authorization rules for the current room version.
///
/// * `fetch_event` - Function to fetch an event in the room given its event ID.
///
/// ## Returns
///
/// Returns the ordered list of event IDs from earliest to latest.
#[instrument(skip_all)]
fn sort_power_events<E: Event>(
    conflicted_power_events: Vec<E::Id>,
    full_conflicted_set: &HashSet<E::Id>,
    rules: &AuthorizationRules,
    fetch_event: impl Fn(&EventId) -> Option<E>,
) -> Result<Vec<E::Id>> {
    debug!("reverse topological sort of power events");

    // A representation of the DAG, a map of event ID to its list of auth events that are in the
    // full conflicted set.
    let mut graph = HashMap::new();

    // Fill the graph.
    for event_id in conflicted_power_events {
        add_event_and_auth_chain_to_graph(&mut graph, event_id, full_conflicted_set, &fetch_event);

        // TODO: if these functions are ever made async here
        // is a good place to yield every once in a while so other
        // tasks can make progress
    }

    // The map of event ID to the power level of the sender of the event.
    let mut event_to_power_level = HashMap::new();
    // We need to know the creator in case of missing power levels. Given that it's the same for all
    // the events in the room, we will just load it for the first event and reuse it.
    let creators_lock = OnceLock::new();

    // Get the power level of the sender of each event in the graph.
    for event_id in graph.keys() {
        let sender_power_level =
            power_level_for_sender(event_id.borrow(), rules, &creators_lock, &fetch_event)
                .map_err(Error::AuthEvent)?;
        debug!(
            event_id = event_id.borrow().as_str(),
            power_level = ?sender_power_level,
            "found the power level of an event's sender",
        );

        event_to_power_level.insert(event_id.clone(), sender_power_level);

        // TODO: if these functions are ever made async here
        // is a good place to yield every once in a while so other
        // tasks can make progress
    }

    reverse_topological_power_sort(&graph, |event_id| {
        let event = fetch_event(event_id).ok_or_else(|| Error::NotFound(event_id.to_owned()))?;
        let power_level = *event_to_power_level
            .get(event_id)
            .ok_or_else(|| Error::NotFound(event_id.to_owned()))?;
        Ok((power_level, event.origin_server_ts()))
    })
}

/// Sorts the given event graph using reverse topological power ordering.
///
/// Definition in the specification:
///
/// > The reverse topological power ordering of a set of events is the lexicographically smallest
/// > topological ordering based on the DAG formed by auth events. The reverse topological power
/// > ordering is ordered from earliest event to latest. For comparing two topological orderings to
/// > determine which is the lexicographically smallest, the following comparison relation on events
/// > is used: for events x and y, x < y if
/// >
/// > 1. x’s sender has greater power level than y’s sender, when looking at their respective
/// > auth_events; or
/// > 2. the senders have the same power level, but x’s origin_server_ts is less than y’s
/// > origin_server_ts; or
/// > 3. the senders have the same power level and the events have the same origin_server_ts, but
/// > x’s event_id is less than y’s event_id.
/// >
/// > The reverse topological power ordering can be found by sorting the events using Kahn’s
/// > algorithm for topological sorting, and at each step selecting, among all the candidate
/// > vertices, the smallest vertex using the above comparison relation.
///
/// ## Arguments
///
/// * `graph` - The graph to sort. A map of event ID to its auth events that are in the full
///   conflicted set.
///
/// * `event_details_fn` - Function to obtain a (power level, origin_server_ts) of an event for
///   breaking ties.
///
/// ## Returns
///
/// Returns the ordered list of event IDs from earliest to latest.
#[instrument(skip_all)]
pub fn reverse_topological_power_sort<Id, F>(
    graph: &HashMap<Id, HashSet<Id>>,
    event_details_fn: F,
) -> Result<Vec<Id>>
where
    F: Fn(&EventId) -> Result<(UserPowerLevel, MilliSecondsSinceUnixEpoch)>,
    Id: Clone + Eq + Ord + Hash + Borrow<EventId>,
{
    #[derive(PartialEq, Eq)]
    struct TieBreaker<'a, Id> {
        power_level: UserPowerLevel,
        origin_server_ts: MilliSecondsSinceUnixEpoch,
        event_id: &'a Id,
    }

    impl<Id> Ord for TieBreaker<'_, Id>
    where
        Id: Ord,
    {
        fn cmp(&self, other: &Self) -> Ordering {
            // NOTE: the power level comparison is "backwards" intentionally.
            other
                .power_level
                .cmp(&self.power_level)
                .then(self.origin_server_ts.cmp(&other.origin_server_ts))
                .then(self.event_id.cmp(other.event_id))
        }
    }

    impl<Id> PartialOrd for TieBreaker<'_, Id>
    where
        Id: Ord,
    {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    // We consider that the DAG is directed from most recent events to oldest events, so an event is
    // an incoming edge to its auth events.

    // Map of event to the list of events in its auth events.
    let mut outgoing_edges_map = graph.clone();

    // Map of event to the list of events that reference it in its auth events.
    let mut incoming_edges_map: HashMap<_, HashSet<_>> = HashMap::new();

    // Vec of events that have an outdegree of zero (no outgoing edges), i.e. the oldest events.
    let mut zero_outdegrees = Vec::new();

    // Populate the list of events with an outdegree of zero, and the map of incoming edges.
    for (event_id, outgoing_edges) in graph {
        if outgoing_edges.is_empty() {
            let (power_level, origin_server_ts) = event_details_fn(event_id.borrow())?;

            // `Reverse` because `BinaryHeap` sorts largest -> smallest and we need
            // smallest -> largest.
            zero_outdegrees.push(Reverse(TieBreaker { power_level, origin_server_ts, event_id }));
        }

        incoming_edges_map.entry(event_id).or_default();

        for auth_event_id in outgoing_edges {
            incoming_edges_map.entry(auth_event_id).or_default().insert(event_id);
        }
    }

    // Use a BinaryHeap to keep the events with an outdegree of zero sorted.
    let mut heap = BinaryHeap::from(zero_outdegrees);
    let mut sorted = vec![];

    // Apply Kahn's algorithm.
    // https://en.wikipedia.org/wiki/Topological_sorting#Kahn's_algorithm
    while let Some(Reverse(item)) = heap.pop() {
        let event_id = item.event_id;

        for &parent_id in incoming_edges_map
            .get(event_id)
            .expect("event ID in heap should also be in incoming edges map")
        {
            let outgoing_edges = outgoing_edges_map
                .get_mut(parent_id.borrow())
                .expect("outgoing edges map should have a key for all event IDs");

            outgoing_edges.remove(event_id.borrow());

            // Push on the heap once all the outgoing edges have been removed.
            if outgoing_edges.is_empty() {
                let (power_level, origin_server_ts) = event_details_fn(parent_id.borrow())?;
                heap.push(Reverse(TieBreaker {
                    power_level,
                    origin_server_ts,
                    event_id: parent_id,
                }));
            }
        }

        sorted.push(event_id.clone());
    }

    Ok(sorted)
}

/// Find the power level for the sender of the event of the given event ID or return a default value
/// of zero.
///
/// We find the most recent `m.room.power_levels` by walking backwards in the auth chain of the
/// event.
///
/// Do NOT use this anywhere but topological sort.
///
/// ## Arguments
///
/// * `event_id` - The event ID of the event to get the power level of the sender of.
///
/// * `rules` - The authorization rules for the current room version.
///
/// * `creator_lock` - A lock used to cache the user ID of the creator of the room. If it is empty
///   the creator will be fetched in the auth chain and used to populate the lock.
///
/// * `fetch_event` - Function to fetch an event in the room given its event ID.
///
/// ## Returns
///
/// Returns the power level of the sender of the event or an `Err(_)` if one of the auth events if
/// malformed.
fn power_level_for_sender<E: Event>(
    event_id: &EventId,
    rules: &AuthorizationRules,
    creators_lock: &OnceLock<HashSet<OwnedUserId>>,
    fetch_event: impl Fn(&EventId) -> Option<E>,
) -> std::result::Result<UserPowerLevel, String> {
    let event = fetch_event(event_id);
    let mut room_create_event = None;
    let mut room_power_levels_event = None;

    if let Some(event) = &event
        && rules.room_create_event_id_as_room_id
        && creators_lock.get().is_none()
    {
        // The m.room.create event is not in the auth events, we can get its ID via the room ID.
        room_create_event = event
            .room_id()
            .and_then(|room_id| room_id.room_create_event_id().ok())
            .and_then(|room_create_event_id| fetch_event(&room_create_event_id));
    }

    for auth_event_id in event.as_ref().map(|pdu| pdu.auth_events()).into_iter().flatten() {
        if let Some(auth_event) = fetch_event(auth_event_id.borrow()) {
            if is_type_and_key(&auth_event, &TimelineEventType::RoomPowerLevels, "") {
                room_power_levels_event = Some(RoomPowerLevelsEvent::new(auth_event));
            } else if !rules.room_create_event_id_as_room_id
                && creators_lock.get().is_none()
                && is_type_and_key(&auth_event, &TimelineEventType::RoomCreate, "")
            {
                room_create_event = Some(auth_event);
            }

            if room_power_levels_event.is_some()
                && (rules.room_create_event_id_as_room_id
                    || creators_lock.get().is_some()
                    || room_create_event.is_some())
            {
                break;
            }
        }
    }

    // TODO: Use OnceLock::try_or_get_init when it is stabilized.
    let creators = if let Some(creators) = creators_lock.get() {
        Some(creators)
    } else if let Some(room_create_event) = room_create_event {
        let room_create_event = RoomCreateEvent::new(room_create_event);
        let creators = room_create_event.creators(rules)?;
        Some(creators_lock.get_or_init(|| creators))
    } else {
        None
    };

    if let Some((event, creators)) = event.zip(creators) {
        room_power_levels_event.user_power_level(event.sender(), creators, rules)
    } else {
        room_power_levels_event
            .get_as_int_or_default(RoomPowerLevelsIntField::UsersDefault, rules)
            .map(Into::into)
    }
}

/// Perform the iterative auth checks to the given list of events.
///
/// Definition in the specification:
///
/// > The iterative auth checks algorithm takes as input an initial room state and a sorted list of
/// > state events, and constructs a new room state by iterating through the event list and applying
/// > the state event to the room state if the state event is allowed by the authorization rules. If
/// > the state event is not allowed by the authorization rules, then the event is ignored. If a
/// > (event_type, state_key) key that is required for checking the authorization rules is not
/// > present in the state, then the appropriate state event from the event’s auth_events is used if
/// > the auth event is not rejected.
///
/// ## Arguments
///
/// * `rules` - The authorization rules for the current room version.
///
/// * `events` - The sorted state events to apply to the `partial_state`.
///
/// * `state` - The current state that was partially resolved for the room.
///
/// * `fetch_event` - Function to fetch an event in the room given its event ID.
///
/// ## Returns
///
/// Returns the partially resolved state, or an `Err(_)` if one of the state events in the room has
/// an unexpected format.
fn iterative_auth_checks<E: Event + Clone>(
    rules: &AuthorizationRules,
    events: &[E::Id],
    mut state: StateMap<E::Id>,
    fetch_event: impl Fn(&EventId) -> Option<E>,
) -> Result<StateMap<E::Id>> {
    debug!("starting iterative auth checks");

    trace!(list = ?events, "events to check");

    for event_id in events {
        let event = fetch_event(event_id.borrow())
            .ok_or_else(|| Error::NotFound(event_id.borrow().to_owned()))?;
        let state_key = event.state_key().ok_or(Error::MissingStateKey)?;

        let mut auth_events = StateMap::new();
        for auth_event_id in event.auth_events() {
            if let Some(auth_event) = fetch_event(auth_event_id.borrow()) {
                if !auth_event.rejected() {
                    auth_events.insert(
                        auth_event
                            .event_type()
                            .with_state_key(auth_event.state_key().ok_or(Error::MissingStateKey)?),
                        auth_event,
                    );
                }
            } else {
                warn!(event_id = %auth_event_id.borrow(), "missing auth event");
            }
        }

        // If the `m.room.create` event is not in the auth events, we need to add it, because it's
        // always part of the state and required in the auth rules.
        if rules.room_create_event_id_as_room_id
            && *event.event_type() != TimelineEventType::RoomCreate
        {
            if let Some(room_create_event) = event
                .room_id()
                .and_then(|room_id| room_id.room_create_event_id().ok())
                .and_then(|room_create_event_id| fetch_event(&room_create_event_id))
            {
                auth_events.insert((StateEventType::RoomCreate, String::new()), room_create_event);
            } else {
                warn!("missing m.room.create event");
            }
        }

        let auth_types = match auth_types_for_event(
            event.event_type(),
            event.sender(),
            Some(state_key),
            event.content(),
            rules,
        ) {
            Ok(auth_types) => auth_types,
            Err(error) => {
                warn!("failed to get list of required auth events for malformed event: {error}");
                continue;
            }
        };

        for key in auth_types {
            if let Some(auth_event_id) = state.get(&key) {
                if let Some(auth_event) = fetch_event(auth_event_id.borrow()) {
                    if !auth_event.rejected() {
                        auth_events.insert(key.to_owned(), auth_event);
                    }
                } else {
                    warn!(event_id = %auth_event_id.borrow(), "missing auth event");
                }
            }
        }

        match check_state_dependent_auth_rules(rules, &event, |ty, key| {
            auth_events.get(&ty.with_state_key(key))
        }) {
            Ok(()) => {
                // Add event to the partially resolved state.
                state.insert(event.event_type().with_state_key(state_key), event_id.clone());
            }
            Err(error) => {
                // Don't add this event to the state.
                warn!("event failed the authentication check: {error}");
            }
        }

        // TODO: if these functions are ever made async here
        // is a good place to yield every once in a while so other
        // tasks can make progress
    }

    Ok(state)
}

/// Perform mainline ordering of the given events.
///
/// Definition in the spec:
///
/// > Given mainline positions calculated from P, the mainline ordering based on P of a set of
/// > events is the ordering, from smallest to largest, using the following comparison relation on
/// > events: for events x and y, x < y if
/// >
/// > 1. the mainline position of x is greater than the mainline position of y (i.e. the auth chain
/// > of x is based on an earlier event in the mainline than y); or
/// > 2. the mainline positions of the events are the same, but x’s origin_server_ts is less than
/// > y’s origin_server_ts; or
/// > 3. the mainline positions of the events are the same and the events have the same
/// > origin_server_ts, but x’s event_id is less than y’s event_id.
///
/// ## Arguments
///
/// * `events` - The list of event IDs to sort.
///
/// * `power_level` - The power level event in the current state.
///
/// * `fetch_event` - Function to fetch an event in the room given its event ID.
///
/// ## Returns
///
/// Returns the sorted list of event IDs, or an `Err(_)` if one the event in the room has an
/// unexpected format.
fn mainline_sort<E: Event>(
    events: &[E::Id],
    mut power_level: Option<E::Id>,
    fetch_event: impl Fn(&EventId) -> Option<E>,
) -> Result<Vec<E::Id>> {
    debug!("mainline sort of events");

    // There are no events to sort, bail.
    if events.is_empty() {
        return Ok(vec![]);
    }

    // Populate the mainline of the power level.
    let mut mainline = vec![];

    while let Some(power_level_event_id) = power_level {
        mainline.push(power_level_event_id.clone());

        let power_level_event = fetch_event(power_level_event_id.borrow())
            .ok_or_else(|| Error::NotFound(power_level_event_id.borrow().to_owned()))?;

        power_level = None;

        for auth_event_id in power_level_event.auth_events() {
            let auth_event = fetch_event(auth_event_id.borrow())
                .ok_or_else(|| Error::NotFound(power_level_event_id.borrow().to_owned()))?;
            if is_type_and_key(&auth_event, &TimelineEventType::RoomPowerLevels, "") {
                power_level = Some(auth_event_id.to_owned());
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
        .map(|(idx, event_id)| ((*event_id).clone(), idx))
        .collect::<HashMap<_, _>>();

    let mut order_map = HashMap::new();
    for event_id in events.iter() {
        if let Some(event) = fetch_event(event_id.borrow())
            && let Ok(position) = mainline_position(event, &mainline_map, &fetch_event)
        {
            order_map.insert(
                event_id,
                (
                    position,
                    fetch_event(event_id.borrow()).map(|event| event.origin_server_ts()),
                    event_id,
                ),
            );
        }

        // TODO: if these functions are ever made async here
        // is a good place to yield every once in a while so other
        // tasks can make progress
    }

    let mut sorted_event_ids = order_map.keys().map(|&k| k.clone()).collect::<Vec<_>>();
    sorted_event_ids.sort_by_key(|event_id| order_map.get(event_id).unwrap());

    Ok(sorted_event_ids)
}

/// Get the mainline position of the given event from the given mainline map.
///
/// Definition in the spec:
///
/// > Let P = P0 be an m.room.power_levels event. Starting with i = 0, repeatedly fetch Pi+1, the
/// > m.room.power_levels event in the auth_events of Pi. Increment i and repeat until Pi has no
/// > m.room.power_levels event in its auth_events. The mainline of P0 is the list of events [P0 ,
/// > P1, … , Pn], fetched in this way.
/// >
/// > Let e = e0 be another event (possibly another m.room.power_levels event). We can compute a
/// > similar list of events [e1, …, em], where ej+1 is the m.room.power_levels event in the
/// > auth_events of ej and where em has no m.room.power_levels event in its auth_events. (Note that
/// > the event we started with, e0, is not included in this list. Also note that it may be empty,
/// > because e may not cite an m.room.power_levels event in its auth_events at all.)
/// >
/// > Now compare these two lists as follows.
/// >
/// > * Find the smallest index j ≥ 1 for which ej belongs to the mainline of P.
/// > * If such a j exists, then ej = Pi for some unique index i ≥ 0. Otherwise set i = ∞, where ∞
/// > is a sentinel value greater than any integer.
/// > * In both cases, the mainline position of e is i.
///
/// ## Arguments
///
/// * `event` - The event to compute the mainline position of.
///
/// * `mainline_map` - The mainline map of the m.room.power_levels event.
///
/// * `fetch_event` - Function to fetch an event in the room given its event ID.
///
/// ## Returns
///
/// Returns the mainline position of the event, or an `Err(_)` if one of the events in the auth
/// chain of the event was not found.
fn mainline_position<E: Event>(
    event: E,
    mainline_map: &HashMap<E::Id, usize>,
    fetch_event: impl Fn(&EventId) -> Option<E>,
) -> Result<usize> {
    let mut current_event = Some(event);

    while let Some(event) = current_event {
        let event_id = event.event_id();
        debug!(event_id = event_id.borrow().as_str(), "mainline");

        // If the current event is in the mainline map, return its position.
        if let Some(position) = mainline_map.get(event_id.borrow()) {
            return Ok(*position);
        }

        current_event = None;

        // Look for the power levels event in the auth events.
        for auth_event_id in event.auth_events() {
            let auth_event = fetch_event(auth_event_id.borrow())
                .ok_or_else(|| Error::NotFound(auth_event_id.borrow().to_owned()))?;

            if is_type_and_key(&auth_event, &TimelineEventType::RoomPowerLevels, "") {
                current_event = Some(auth_event);
                break;
            }
        }
    }

    // Did not find a power level event so we default to zero.
    Ok(0)
}

/// Add the event with the given event ID and all the events in its auth chain that are in the full
/// conflicted set to the graph.
fn add_event_and_auth_chain_to_graph<E: Event>(
    graph: &mut HashMap<E::Id, HashSet<E::Id>>,
    event_id: E::Id,
    full_conflicted_set: &HashSet<E::Id>,
    fetch_event: impl Fn(&EventId) -> Option<E>,
) {
    let mut state = vec![event_id];

    // Iterate through the auth chain of the event.
    while let Some(event_id) = state.pop() {
        // Add the current event to the graph.
        graph.entry(event_id.clone()).or_default();

        // Iterate through the auth events of this event.
        for auth_event_id in fetch_event(event_id.borrow())
            .as_ref()
            .map(|event| event.auth_events())
            .into_iter()
            .flatten()
        {
            // If the auth event ID is in the full conflicted set…
            if full_conflicted_set.contains(auth_event_id.borrow()) {
                // If the auth event ID is not in the graph, we need to check its auth events later.
                if !graph.contains_key(auth_event_id.borrow()) {
                    state.push(auth_event_id.to_owned());
                }

                // Add the auth event ID to the list of incoming edges.
                graph.get_mut(event_id.borrow()).unwrap().insert(auth_event_id.to_owned());
            }
        }
    }
}

/// Whether the given event ID belongs to a power event.
///
/// See the docs of `is_power_event()` for the definition of a power event.
fn is_power_event_id<E: Event>(event_id: &EventId, fetch: impl Fn(&EventId) -> Option<E>) -> bool {
    match fetch(event_id).as_ref() {
        Some(state) => is_power_event(state),
        _ => false,
    }
}

fn is_type_and_key(event: impl Event, event_type: &TimelineEventType, state_key: &str) -> bool {
    event.event_type() == event_type && event.state_key() == Some(state_key)
}

/// Whether the given event is a power event.
///
/// Definition in the spec:
///
/// > A power event is a state event with type `m.room.power_levels` or `m.room.join_rules`, or a
/// > state event with type `m.room.member` where the `membership` is `leave` or `ban` and the
/// > `sender` does not match the `state_key`. The idea behind this is that power events are events
/// > that might remove someone’s ability to do something in the room.
fn is_power_event(event: impl Event) -> bool {
    match event.event_type() {
        TimelineEventType::RoomPowerLevels
        | TimelineEventType::RoomJoinRules
        | TimelineEventType::RoomCreate => event.state_key() == Some(""),
        TimelineEventType::RoomMember => {
            let room_member_event = RoomMemberEvent::new(event);
            if room_member_event.membership().is_ok_and(|membership| {
                matches!(membership, MembershipState::Leave | MembershipState::Ban)
            }) {
                return Some(room_member_event.sender().as_str()) != room_member_event.state_key();
            }

            false
        }
        _ => false,
    }
}

/// Convenience trait for adding event type plus state key to state maps.
pub(crate) trait EventTypeExt {
    fn with_state_key(self, state_key: impl Into<String>) -> (StateEventType, String);
}

impl EventTypeExt for StateEventType {
    fn with_state_key(self, state_key: impl Into<String>) -> (StateEventType, String) {
        (self, state_key.into())
    }
}

impl EventTypeExt for TimelineEventType {
    fn with_state_key(self, state_key: impl Into<String>) -> (StateEventType, String) {
        (self.to_string().into(), state_key.into())
    }
}

impl<T> EventTypeExt for &T
where
    T: EventTypeExt + Clone,
{
    fn with_state_key(self, state_key: impl Into<String>) -> (StateEventType, String) {
        self.to_owned().with_state_key(state_key)
    }
}
