use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap},
    error::Error,
    fs,
    ops::Deref,
    path::Path,
    sync::LazyLock,
};

use ruma_common::{
    EventId, MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId, OwnedUserId, RoomId, UserId,
    room_version_rules::{AuthorizationRules, StateResolutionV2Rules},
};
use ruma_events::{StateEventType, TimelineEventType};
use ruma_state_res::{
    Event, StateMap, events::RoomCreateEvent, resolve, utils::event_id_set::EventIdSet,
};
use serde::{Deserialize, Serialize};
use serde_json::{
    from_str as from_json_str, to_string_pretty as to_json_string_pretty,
    value::RawValue as RawJsonValue,
};
use similar::{Algorithm, udiff::unified_diff};

static FIXTURES_PATH: LazyLock<&'static Path> =
    LazyLock::new(|| Path::new("tests/it/resolve/fixtures"));

/// Create a snapshot test attempting the state resolution of several batches of PDUs.
///
/// State resolution is performed:
///
/// * Iteratively by PDU.
/// * Iteratively by batch.
/// * Atomically on the full list of PDUs at once.
///
/// # Arguments
///
/// * `name` - The test function's name.
/// * `pdus_paths` - A list of JSON files relative to `tests/it/fixtures/resolve` containing each a
///   batch of PDUs.
///
/// # Panics
///
/// Panics if one of the `auth_events` of a PDU is missing, if the three final resolved states don't
/// match, or if the final resolved state doesn't match the snapshot.
macro_rules! snapshot_test_batches {
    ($name:ident, $pdus_paths:expr $(,)?) => {
        #[test_log::test]
        fn $name() {
            let resolved_state = crate::resolve::macros::test_resolve_batches(&$pdus_paths);

            insta::with_settings!({
                description => "Resolved state",
                omit_expression => true,
                snapshot_path => "resolve/snapshots",
                prepend_module_to_snapshot => false,
                snapshot_suffix => "resolved_state",
            }, {
                insta::assert_snapshot!(resolved_state);
            });
        }
    };
}

/// Create a snapshot test attempting the state resolution of several state maps.
///
/// # Arguments
///
/// * `name` - The test function's name.
/// * `state_maps_paths` - A list of JSON files relative to `tests/it/fixtures/resolve` containing
///   each the list of event IDs forming a state map.
/// * `pdus_paths` - A list of JSON files relative to `tests/it/fixtures/resolve` containing each a
///   batch of PDUs.
///
/// # Panics
///
/// Panics if one of the `auth_events` of a PDU is missing or if the resolved state doesn't match
/// the snapshot.
macro_rules! snapshot_test_state_maps {
    ($name:ident, $state_maps_paths:expr, $pdus_paths:expr $(,)?) => {
        #[test_log::test]
        fn $name() {
            let resolved_state = crate::resolve::macros::test_resolve_state_maps(&$state_maps_paths, &$pdus_paths);

            insta::with_settings!({
                description => "Resolved state",
                omit_expression => true,
                snapshot_path => "resolve/snapshots",
                prepend_module_to_snapshot => false,
                snapshot_suffix => "resolved_state",
            }, {
                insta::assert_snapshot!(resolved_state);
            });
        }
    };
}

/// Asserts that two strings are equal and prints a diff if they are not.
///
/// # Arguments
///
/// * `lhs_name` - The name for the left-hand side string.
/// * `lhs` - The left-hand side string.
/// * `rhs_name` - The name for the right-hand side string.
/// * `rhs` - The right-hand side string.
macro_rules! assert_eq_diff {
    ($lhs_name:literal => $lhs:expr, $rhs_name:literal => $rhs:expr $(,)?) => {
        if $lhs != $rhs {
            let diff =
                unified_diff(Algorithm::default(), &$lhs, &$rhs, 3, Some(($lhs_name, $rhs_name)));

            panic!("Assertion {} == {} failed:\n{diff}", $lhs_name, $rhs_name);
        }
    };
}

/// Test state resolution of several batches of PDUs.
///
/// State resolution is performed:
///
/// * Iteratively by PDU.
/// * Iteratively by batch.
/// * Atomically on the full list of PDUs at once.
///
/// # Arguments
///
/// * `pdus_paths` - A list of JSON files relative to `tests/it/fixtures/resolve` containing each a
///   batch of PDUs.
///
/// # Returns
///
/// Returns the pretty-printed JSON serialization of the resolved state.
///
/// # Panics
///
/// Panics if one of the `auth_events` of a PDU is missing, or if the three final resolved states
/// don't match.
pub(super) fn test_resolve_batches(pdus_paths: &[&str]) -> String {
    let (pdu_batches, auth_rules, state_res_rules) = load_pdus_and_room_version_rules(pdus_paths);

    // Resolve PDUs iteratively, using the ordering of `prev_events`.
    let iteratively_resolved_state = resolve_iteratively(
        pdu_batches.iter().flat_map(|x| x.iter()),
        &auth_rules,
        &state_res_rules,
    )
    .expect("iterative state resolution should succeed");

    // Resolve PDUs in batches by file.
    let mut pdus_map = HashMap::new();
    let mut batched_resolved_state = None;
    for pdus in &pdu_batches {
        batched_resolved_state = Some(
            resolve_batch(
                pdus,
                &auth_rules,
                &state_res_rules,
                &mut pdus_map,
                batched_resolved_state,
            )
            .expect("batched state resolution step should succeed"),
        );
    }
    let batched_resolved_state =
        batched_resolved_state.expect("batched state resolution should have run at least once");

    // Resolve all PDUs in a single step.
    let atomic_resolved_state = resolve_batch(
        pdu_batches.iter().flat_map(|x| x.iter()),
        &auth_rules,
        &state_res_rules,
        &mut HashMap::new(),
        None,
    )
    .expect("atomic state resolution should succeed");

    let iteratively_resolved_state =
        state_map_to_json_string(iteratively_resolved_state, &pdus_map);
    let batched_resolved_state = state_map_to_json_string(batched_resolved_state, &pdus_map);
    let atomic_resolved_state = state_map_to_json_string(atomic_resolved_state, &pdus_map);

    assert_eq_diff!(
        "iterative" => iteratively_resolved_state,
        "batched" => batched_resolved_state,
    );
    assert_eq_diff!(
        "batched" => batched_resolved_state,
        "atomic" => atomic_resolved_state,
    );

    iteratively_resolved_state
}

/// Test state resolution of several state maps.
///
/// # Arguments
///
/// * `state_maps_paths` - A list of JSON files relative to `tests/it/fixtures/resolve` containing
///   each the list of event IDs forming a state map.
/// * `pdus_paths` - A list of JSON files relative to `tests/it/fixtures/resolve` containing each a
///   batch of PDUs.
///
/// # Returns
///
/// Returns the pretty-printed JSON serialization of the resolved state.
///
/// # Panics
///
/// Panics if one of the `auth_events` of a PDU is missing.
pub(super) fn test_resolve_state_maps(state_maps_paths: &[&str], pdus_paths: &[&str]) -> String {
    let (pdu_batches, auth_rules, state_res_rules) = load_pdus_and_room_version_rules(pdus_paths);

    let pdus = pdu_batches.into_iter().flat_map(|x| x.into_iter()).collect::<Vec<_>>();
    let pdus_map: HashMap<OwnedEventId, Pdu> =
        pdus.clone().into_iter().map(|pdu| (pdu.event_id().to_owned(), pdu.to_owned())).collect();

    let state_maps = load_state_maps(state_maps_paths, &pdus_map);

    let mut auth_chain_sets = Vec::new();
    for state_map in &state_maps {
        let mut auth_chain = EventIdSet::new();

        for event_id in state_map.values() {
            let pdu = pdus_map
                .get(event_id)
                .expect("We already confirmed all state set event ids have pdus");

            auth_chain.extend(pdu_auth_chain(pdu, &pdus_map));
        }

        auth_chain_sets.push(auth_chain);
    }

    let resolved_state = resolve(
        &auth_rules,
        &state_res_rules,
        &state_maps,
        auth_chain_sets,
        |x| pdus_map.get(x).cloned(),
        |conflicted_state_set| conflicted_state_subgraph(conflicted_state_set, &pdus_map),
    )
    .expect("atomic state resolution should succeed");

    state_map_to_json_string(resolved_state, &pdus_map)
}

/// Load PDUs from JSON files and extract the room version rules.
///
/// The room version rules are determined from the first event in the first JSON file, which must be
/// an `m.room.create` event with a supported `room_version`.
///
/// # Arguments
///
/// * `pdus_paths` - A list of JSON files relative to `tests/it/fixtures/resolve` containing each an
///   array of PDUs.
fn load_pdus_and_room_version_rules(
    pdus_paths: &[&str],
) -> (Vec<Vec<Pdu>>, AuthorizationRules, StateResolutionV2Rules) {
    let pdu_batches = pdus_paths
        .iter()
        .map(|x| {
            from_json_str(
                &fs::read_to_string(FIXTURES_PATH.join(x))
                    .expect("should be able to read JSON file of PDUs"),
            )
            .expect("should be able to deserialize JSON file of PDUs")
        })
        .collect::<Vec<Vec<Pdu>>>();

    let room_create = pdu_batches
        .first()
        .expect("there should be at least one JSON file of PDUs")
        .first()
        .expect("there should be at least one PDU in the first JSON file");

    assert_eq!(
        room_create.event_type,
        TimelineEventType::RoomCreate,
        "first PDU in first JSON file should be an `m.room.create` event",
    );

    let room_version_id = RoomCreateEvent::new(room_create)
        .room_version()
        .expect("`m.room.create` PDU's content should be valid");
    let rules = room_version_id.rules().expect("room version should be supported");
    let auth_rules = rules.authorization;
    let state_res_rules =
        rules.state_res.v2_rules().expect("resolve only supports state resolution version 2");

    (pdu_batches, auth_rules, *state_res_rules)
}

/// Load state maps from JSON files.
///
/// # Arguments
///
/// * `state_maps_paths` - A list of JSON files relative to `tests/it/fixtures/resolve` containing
///   each the list of event IDs forming a state map.
/// * `pdus_map` - A map containing the PDUs referenced in the state maps, used to get their `type`
///   and `state_key`.
fn load_state_maps(
    state_maps_paths: &[&str],
    pdus_map: &HashMap<OwnedEventId, Pdu>,
) -> Vec<StateMap<OwnedEventId>> {
    state_maps_paths
        .iter()
        .map(|path| {
            from_json_str::<Vec<OwnedEventId>>(
                &fs::read_to_string(FIXTURES_PATH.join(path))
                    .expect("should be able to read JSON file of event IDs"),
            )
            .expect("should be able to deserialize JSON file of event IDs")
            .into_iter()
            .map(|event_id| {
                pdus_map
                    .get(&event_id)
                    .map(|pdu| {
                        (
                            (
                                pdu.event_type().to_string().into(),
                                pdu.state_key.clone().expect("All PDUs must be state events"),
                            ),
                            event_id,
                        )
                    })
                    .expect("Event IDs in state set JSON file must be in PDUs JSON")
            })
            .collect()
        })
        .collect()
}

/// Perform state resolution on a batch of PDUs.
///
/// This function can be used to resolve the state of a room in a single call if all PDUs are
/// provided at once, or across multiple calls if given PDUs in batches in a loop. The latter form
/// simulates the case commonly experienced by homeservers during normal operation.
///
/// # Arguments
///
/// * `pdus`: An iterator of [`Pdu`]s to resolve, either alone or against the `prev_state`.
/// * `auth_rules`: The authorization rules of the room version.
/// * `state_res_rules`: The state resolution rules of the room version.
/// * `pdus_map`: A map of [`OwnedEventId`] to the [`Pdu`] with that ID. This is populated by this
///   function and should not be mutated outside of this function. Should be empty for the first
///   call.
/// * `prev_state`: The state returned by a previous call to this function, if any. Should be `None`
///   for the first call.
fn resolve_batch<'a, I>(
    pdus: I,
    auth_rules: &AuthorizationRules,
    state_res_rules: &StateResolutionV2Rules,
    pdus_map: &mut HashMap<OwnedEventId, Pdu>,
    prev_state: Option<StateMap<OwnedEventId>>,
) -> Result<StateMap<OwnedEventId>, Box<dyn Error>>
where
    I: IntoIterator<Item = &'a Pdu> + Clone,
{
    let mut state_maps = prev_state.into_iter().collect::<Vec<_>>();

    for pdu in pdus.clone() {
        // Insert each state event into its own StateMap because we don't know any valid groupings.
        let mut state_map = StateMap::new();
        state_map.insert(
            (
                pdu.event_type().to_string().into(),
                pdu.state_key().ok_or("all PDUs should be state events")?.to_owned(),
            ),
            pdu.event_id().clone(),
        );

        state_maps.push(state_map);
    }

    pdus_map
        .extend(pdus.clone().into_iter().map(|pdu| (pdu.event_id().to_owned(), pdu.to_owned())));

    let mut auth_chain_sets = Vec::new();
    for pdu in pdus {
        auth_chain_sets.push(pdu_auth_chain(pdu, pdus_map));
    }

    resolve(
        auth_rules,
        state_res_rules,
        &state_maps,
        auth_chain_sets,
        |x| pdus_map.get(x).cloned(),
        |conflicted_state_set| conflicted_state_subgraph(conflicted_state_set, pdus_map),
    )
    .map_err(Into::into)
}

/// Perform state resolution on a batch of PDUs iteratively, one-by-one.
///
/// This function walks the `prev_events` of each PDU forward, resolving each pdu against the
/// state(s) of it's `prev_events`, to emulate what would happen in a regular room a server is
/// participating in.
///
/// # Arguments
///
/// * `pdus`: An iterator of [`Pdu`]s to resolve, with the following assumptions:
///   * `prev_events` of each PDU points to another provided state event.
/// * `auth_rules`: The authorization rules of the room version.
/// * `state_res_rules`: The state resolution rules of the room version.
///
/// # Returns
///
/// The state resolved by resolving all the leaves (PDUs which don't have any other PDUs pointing
/// to it via `prev_events`).
fn resolve_iteratively<'a, I>(
    pdus: I,
    auth_rules: &AuthorizationRules,
    state_res_rules: &StateResolutionV2Rules,
) -> Result<StateMap<OwnedEventId>, Box<dyn Error>>
where
    I: IntoIterator<Item = &'a Pdu> + Clone,
{
    let mut forward_prev_events_graph: HashMap<&_, Vec<_>> = HashMap::new();
    let mut stack = Vec::new();

    for pdu in pdus.clone() {
        let mut has_prev_events = false;
        for prev_event in pdu.prev_events() {
            forward_prev_events_graph.entry(prev_event).or_default().push(pdu.event_id());
            has_prev_events = true;
        }
        if pdu.event_type() == &TimelineEventType::RoomCreate && !has_prev_events {
            stack.push(pdu.event_id().to_owned());
        }
    }

    let pdus_map: HashMap<OwnedEventId, Pdu> =
        HashMap::from_iter(pdus.into_iter().map(|pdu| (pdu.event_id().to_owned(), pdu.to_owned())));

    let auth_chain_from_state_map =
        |state_map: &StateMap<OwnedEventId>| -> Result<_, Box<dyn Error>> {
            let mut auth_chain_sets = EventIdSet::new();

            for event_id in state_map.values() {
                let pdu = pdus_map.get(event_id).expect("every pdu should be available");
                auth_chain_sets.extend(pdu_auth_chain(pdu, &pdus_map));
            }

            Ok(auth_chain_sets)
        };

    let mut state_at_events: HashMap<OwnedEventId, StateMap<OwnedEventId>> = HashMap::new();
    let mut leaves = Vec::new();

    'outer: while let Some(event_id) = stack.pop() {
        let mut states_before_event = Vec::new();
        let mut auth_chains_before_event = Vec::new();

        let current_pdu = pdus_map.get(&event_id).expect("every pdu should be available");

        for prev_event in current_pdu.prev_events() {
            let Some(state_at_event) = state_at_events.get(prev_event) else {
                // State for a prev event is not known, we will come back to this event on a later
                // iteration.
                continue 'outer;
            };
            let auth_chain_at_event = auth_chain_from_state_map(state_at_event)?;

            states_before_event.push(state_at_event.clone());
            auth_chains_before_event.push(auth_chain_at_event);
        }

        let state_before_event = resolve(
            auth_rules,
            state_res_rules,
            &states_before_event,
            auth_chains_before_event.clone(),
            |x| pdus_map.get(x).cloned(),
            |conflicted_state_set| conflicted_state_subgraph(conflicted_state_set, &pdus_map),
        )?;

        let auth_chain_before_event = auth_chain_from_state_map(&state_before_event)?;

        let mut proposed_state_at_event = state_before_event.clone();
        proposed_state_at_event.insert(
            (
                current_pdu.event_type().to_string().into(),
                current_pdu.state_key().expect("all pdus are state events").to_owned(),
            ),
            event_id.to_owned(),
        );

        let mut auth_chain_at_event = auth_chain_before_event.clone();
        auth_chain_at_event.extend(pdu_auth_chain(current_pdu, &pdus_map));

        let state_at_event = resolve(
            auth_rules,
            state_res_rules,
            &[state_before_event, proposed_state_at_event],
            vec![auth_chain_before_event, auth_chain_at_event],
            |x| pdus_map.get(x).cloned(),
            |conflicted_state_set| conflicted_state_subgraph(conflicted_state_set, &pdus_map),
        )?;

        state_at_events.insert(event_id.clone(), state_at_event);

        if let Some(prev_events) = forward_prev_events_graph.get(&event_id) {
            stack.extend(prev_events.iter().map(Deref::deref).cloned());
        } else {
            // pdu is a leaf: no `prev_events` point to it.
            leaves.push(event_id);
        }
    }

    if state_at_events.len() != pdus_map.len() {
        panic!(
            "Not all events have a state calculated! This is likely due to an \
             event having a `prev_events` which points to a non-existent PDU."
        );
    }

    let mut leaf_states = Vec::new();
    let mut auth_chain_sets = Vec::new();

    for leaf in leaves {
        let state_at_event = state_at_events.get(&leaf).expect("states at all events are known");
        let auth_chain_at_event = auth_chain_from_state_map(state_at_event)?;

        leaf_states.push(state_at_event.clone());
        auth_chain_sets.push(auth_chain_at_event);
    }

    resolve(
        auth_rules,
        state_res_rules,
        &leaf_states,
        auth_chain_sets,
        |x| pdus_map.get(x).cloned(),
        |conflicted_state_set| conflicted_state_subgraph(conflicted_state_set, &pdus_map),
    )
    .map_err(Into::into)
}

/// Compute the auth chain of the given PDU.
///
/// This walks recursively the `auth_events` starting from the given PDU and using the given map and
/// returns all the event IDs encountered.
///
/// # Panic
///
/// Panics if `pdus_map` does not contain a PDU that appears in the auth chain of `pdu`.
fn pdu_auth_chain(pdu: &Pdu, pdus_map: &HashMap<OwnedEventId, Pdu>) -> EventIdSet<OwnedEventId> {
    let mut auth_chain = EventIdSet::new();
    let mut stack = pdu.auth_events().cloned().collect::<Vec<_>>();

    while let Some(event_id) = stack.pop() {
        if auth_chain.contains(&event_id) {
            continue;
        }

        let Some(pdu) = pdus_map.get(&event_id) else {
            panic!("missing required PDU: {event_id}");
        };

        stack.extend(pdu.auth_events().cloned());
        auth_chain.insert(event_id);
    }

    auth_chain
}

/// Construct the conflicted state subgraph for the given conflicted state set.
fn conflicted_state_subgraph(
    conflicted_state_set: &StateMap<Vec<OwnedEventId>>,
    pdus_map: &HashMap<OwnedEventId, Pdu>,
) -> Option<EventIdSet<OwnedEventId>> {
    let conflicted_event_ids: EventIdSet<_> =
        conflicted_state_set.values().flatten().cloned().collect();
    let mut conflicted_state_subgraph = EventIdSet::new();

    let mut stack = vec![conflicted_event_ids.iter().cloned().collect::<Vec<_>>()];
    let mut path = Vec::new();

    let mut seen_events = EventIdSet::new();

    let next_event = |stack: &mut Vec<Vec<_>>, path: &mut Vec<_>| {
        while stack.last().is_some_and(|s| s.is_empty()) {
            stack.pop();
            path.pop();
        }

        stack.last_mut().and_then(|s| s.pop())
    };

    while let Some(event_id) = next_event(&mut stack, &mut path) {
        path.push(event_id.clone());

        if conflicted_state_subgraph.contains(&event_id) {
            // If we reach a conflicted state subgraph path, this path must also be part of
            // the conflicted state subgraph, as we will eventually reach a conflicted event
            // if we follow this path.
            //
            // We check if path > 1 here and below, as we don't consider a single conflicted
            // event to be a path from one conflicted to another.
            if path.len() > 1 {
                conflicted_state_subgraph.extend(path.iter().cloned());
            }

            // All possible paths from this event must have been traversed in the iteration
            // that caused this event to be added to the conflicted state subgraph in the first
            // place.
            //
            // We pop the path here and below as it won't be removed by `next_event`, due to us
            // never pushing it's auth events to the stack.
            path.pop();
            continue;
        }

        if conflicted_event_ids.contains(&event_id) && path.len() > 1 {
            conflicted_state_subgraph.extend(path.iter().cloned());
        }

        if seen_events.contains(&event_id) {
            // All possible paths from this event must have been traversed in the iteration
            // that caused this event to be added to the conflicted state subgraph in the first
            // place.
            path.pop();
            continue;
        }

        stack.push(pdus_map.get(&event_id)?.auth_events().cloned().collect());

        seen_events.insert(event_id);
    }

    Some(conflicted_state_subgraph)
}

/// A persistent data unit.
#[derive(Deserialize, Clone)]
struct Pdu {
    event_id: OwnedEventId,
    room_id: Option<OwnedRoomId>,
    sender: OwnedUserId,
    origin_server_ts: MilliSecondsSinceUnixEpoch,
    #[serde(rename = "type")]
    event_type: TimelineEventType,
    content: Box<RawJsonValue>,
    state_key: Option<String>,
    prev_events: Vec<OwnedEventId>,
    auth_events: Vec<OwnedEventId>,
    redacts: Option<OwnedEventId>,
    #[serde(default)]
    rejected: bool,
}

impl Event for Pdu {
    type Id = OwnedEventId;

    fn event_id(&self) -> &Self::Id {
        &self.event_id
    }

    fn room_id(&self) -> Option<&RoomId> {
        self.room_id.as_deref()
    }

    fn sender(&self) -> &UserId {
        &self.sender
    }

    fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch {
        self.origin_server_ts
    }

    fn event_type(&self) -> &TimelineEventType {
        &self.event_type
    }

    fn content(&self) -> &RawJsonValue {
        &self.content
    }

    fn state_key(&self) -> Option<&str> {
        self.state_key.as_deref()
    }

    fn prev_events(&self) -> Box<dyn DoubleEndedIterator<Item = &Self::Id> + '_> {
        Box::new(self.prev_events.iter())
    }

    fn auth_events(&self) -> Box<dyn DoubleEndedIterator<Item = &Self::Id> + '_> {
        Box::new(self.auth_events.iter())
    }

    fn redacts(&self) -> Option<&Self::Id> {
        self.redacts.as_ref()
    }

    fn rejected(&self) -> bool {
        self.rejected
    }
}

/// Type describing a resolved state event.
#[derive(Serialize)]
struct ResolvedStateEvent<'a> {
    #[serde(rename = "type")]
    event_type: &'a StateEventType,
    state_key: &'a str,
    event_id: &'a EventId,

    // Ignored in `PartialEq` and `Ord` because we don't want to consider it while sorting.
    content: &'a RawJsonValue,
}

impl PartialEq for ResolvedStateEvent<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.event_type == other.event_type && self.state_key == other.state_key
    }
}

impl Eq for ResolvedStateEvent<'_> {}

impl Ord for ResolvedStateEvent<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.event_type.cmp(other.event_type).then_with(|| self.state_key.cmp(other.state_key))
    }
}

impl PartialOrd for ResolvedStateEvent<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Serialize the given [`StateMap`] to a pretty-printed JSON string.
///
/// This prints the state map as a JSON array of PDUs sorted by event type and state key. The PDUs
/// are pretty printed using a simplified format containing only the `event_id`, `type`, `state_key`
/// and `content` fields.
fn state_map_to_json_string(
    state_map: StateMap<OwnedEventId>,
    pdus_map: &HashMap<OwnedEventId, Pdu>,
) -> String {
    let resolved_state = state_map
        .iter()
        .map(|((event_type, state_key), event_id)| ResolvedStateEvent {
            event_type,
            state_key,
            content: &pdus_map[event_id].content,
            event_id,
        })
        .collect::<BTreeSet<_>>();

    to_json_string_pretty(&resolved_state).expect("resolved state serialization should succeed")
}
