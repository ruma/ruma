//! State resolution integration tests.

use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap, HashSet},
    error::Error,
    fs,
    ops::Deref,
    path::Path,
    sync::LazyLock,
};

use ruma_common::{
    EventId, MilliSecondsSinceUnixEpoch, RoomId, RoomVersionId, UserId,
    room_version_rules::{AuthorizationRules, StateResolutionV2Rules},
};
use ruma_events::{StateEventType, TimelineEventType};
use ruma_state_res::{Event, StateMap, resolve};
use serde::{Deserialize, Serialize};
use serde_json::{
    Error as JsonError, Value as JsonValue, from_str as from_json_str,
    to_string_pretty as to_json_string_pretty, to_value as to_json_value,
    value::RawValue as RawJsonValue,
};
use similar::{Algorithm, udiff::unified_diff};

static FIXTURES_PATH: LazyLock<&'static Path> = LazyLock::new(|| Path::new("tests/it/fixtures"));

/// Create a new snapshot test.
///
/// # Arguments
///
/// * The test function's name.
/// * A list of JSON files relative to `tests/it/fixtures` to load PDUs to resolve from.
macro_rules! snapshot_test {
    ($name:ident, $paths:expr $(,)?) => {
        #[test_log::test]
        fn $name() {
            let crate::resolve::Snapshots {
                resolved_state,
            } = crate::resolve::test_resolve(&$paths);

            insta::with_settings!({
                description => "Resolved state",
                omit_expression => true,
                snapshot_suffix => "resolved_state",
            }, {
                insta::assert_json_snapshot!(&resolved_state);
            });
        }
    };
}

/// Create a new snapshot test, attempting to resolve multiple contrived states.
///
/// # Arguments
///
/// * The test function's name.
/// * A list of JSON files relative to `tests/it/fixtures` to load PDUs to resolve from.
/// * A list of JSON files relative to `tests/it/fixtures` to load event IDs forming contrived
///   states to resolve.
macro_rules! snapshot_test_contrived_states {
    ($name:ident, $pdus_path:expr, $state_set_paths:expr $(,)?) => {
        #[test_log::test]
        fn $name() {
            let crate::resolve::Snapshots {
                resolved_state,
            } = crate::resolve::test_contrived_states(&$pdus_path, &$state_set_paths);

            insta::with_settings!({
                description => "Resolved state",
                omit_expression => true,
                snapshot_suffix => "resolved_state",
            }, {
                insta::assert_json_snapshot!(&resolved_state);
            });
        }
    };
}

// This module must be defined lexically after the `snapshot_test` macro.
mod snapshot_tests;

/// A persistent data unit.
#[derive(Deserialize, Clone)]
struct Pdu {
    event_id: EventId,
    room_id: Option<RoomId>,
    sender: UserId,
    origin_server_ts: MilliSecondsSinceUnixEpoch,
    #[serde(rename = "type")]
    kind: TimelineEventType,
    content: Box<RawJsonValue>,
    state_key: Option<String>,
    prev_events: Vec<EventId>,
    auth_events: Vec<EventId>,
    redacts: Option<EventId>,
    #[serde(default)]
    rejected: bool,
}

impl Event for Pdu {
    type Id = EventId;

    fn event_id(&self) -> &Self::Id {
        &self.event_id
    }

    fn room_id(&self) -> Option<&RoomId> {
        self.room_id.as_ref()
    }

    fn sender(&self) -> &UserId {
        &self.sender
    }

    fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch {
        self.origin_server_ts
    }

    fn event_type(&self) -> &TimelineEventType {
        &self.kind
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

/// Extract `.content.room_version` from a PDU.
#[derive(Deserialize)]
struct ExtractRoomVersion {
    room_version: RoomVersionId,
}

/// Type describing a resolved state event.
#[derive(Serialize)]
struct ResolvedStateEvent {
    kind: StateEventType,
    state_key: String,
    event_id: EventId,

    // Ignored in `PartialEq` and `Ord` because we don't want to consider it while sorting.
    content: JsonValue,
}

impl PartialEq for ResolvedStateEvent {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.state_key == other.state_key
            && self.event_id == other.event_id
    }
}

impl Eq for ResolvedStateEvent {}

impl Ord for ResolvedStateEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        Ordering::Equal
            .then(self.kind.cmp(&other.kind))
            .then(self.state_key.cmp(&other.state_key))
            .then(self.event_id.cmp(&other.event_id))
    }
}

impl PartialOrd for ResolvedStateEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Information to be captured in snapshot assertions
struct Snapshots {
    /// The resolved state of the room.
    resolved_state: BTreeSet<ResolvedStateEvent>,
}

fn snapshot_test_prelude(
    paths: &[&str],
) -> (Vec<Vec<Pdu>>, AuthorizationRules, StateResolutionV2Rules) {
    let pdu_batches = paths
        .iter()
        .map(|x| {
            from_json_str(
                &fs::read_to_string(FIXTURES_PATH.join(x))
                    .expect("should be able to read JSON file of PDUs"),
            )
            .expect("should be able to deserialize JSON file of PDUs")
        })
        .collect::<Vec<Vec<Pdu>>>();

    let room_version_id = {
        let first_pdu = pdu_batches
            .first()
            .expect("there should be at least one file of PDUs")
            .first()
            .expect("there should be at least one PDU in the first file");

        assert_eq!(
            first_pdu.kind,
            TimelineEventType::RoomCreate,
            "the first PDU in the first file should be an m.room.create event",
        );

        from_json_str::<ExtractRoomVersion>(first_pdu.content.get())
            .expect("the m.room.create PDU's content should be valid")
            .room_version
    };
    let rules = room_version_id.rules().expect("room version should be supported");
    let auth_rules = rules.authorization;
    let state_res_rules =
        rules.state_res.v2_rules().expect("resolve only supports state resolution version 2");

    (pdu_batches, auth_rules, *state_res_rules)
}

/// Reshape the data a bit to make the diff and snapshots easier to compare.
fn reshape(
    pdus_by_id: &HashMap<EventId, Pdu>,
    x: StateMap<EventId>,
) -> Result<BTreeSet<ResolvedStateEvent>, JsonError> {
    x.into_iter()
        .map(|((kind, state_key), event_id)| {
            Ok(ResolvedStateEvent {
                kind,
                state_key,
                content: to_json_value(pdus_by_id[&event_id].content())?,
                event_id,
            })
        })
        .collect()
}

/// Test a list of JSON files containing a list of PDUs and return the results.
///
/// State resolution is run both atomically for all PDUs and in batches of PDUs by file.
fn test_resolve(paths: &[&str]) -> Snapshots {
    let (pdu_batches, auth_rules, state_res_rules) = snapshot_test_prelude(paths);

    // Resolve PDUs iteratively, using the ordering of `prev_events`.
    let iteratively_resolved_state = resolve_iteratively(
        &auth_rules,
        &state_res_rules,
        pdu_batches.iter().flat_map(|x| x.iter()),
    )
    .expect("iterative state resolution should succeed");

    // Resolve PDUs in batches by file
    let mut pdus_by_id = HashMap::new();
    let mut batched_resolved_state = None;
    for pdus in &pdu_batches {
        batched_resolved_state = Some(
            resolve_batch(
                &auth_rules,
                &state_res_rules,
                pdus,
                &mut pdus_by_id,
                &mut batched_resolved_state,
            )
            .expect("batched state resolution step should succeed"),
        );
    }
    let batched_resolved_state =
        batched_resolved_state.expect("batched state resolution should have run at least once");

    // Resolve all PDUs in a single step
    let atomic_resolved_state = resolve_batch(
        &auth_rules,
        &state_res_rules,
        pdu_batches.iter().flat_map(|x| x.iter()),
        &mut HashMap::new(),
        &mut None,
    )
    .expect("atomic state resolution should succeed");

    let iteratively_resolved_state = reshape(&pdus_by_id, iteratively_resolved_state)
        .expect("should be able to reshape iteratively resolved state");
    let batched_resolved_state = reshape(&pdus_by_id, batched_resolved_state)
        .expect("should be able to reshape batched resolved state");
    let atomic_resolved_state = reshape(&pdus_by_id, atomic_resolved_state)
        .expect("should be able to reshape atomic resolved state");

    let assert_states_match = |first_resolved_state: &BTreeSet<ResolvedStateEvent>,
                               second_resolved_state: &BTreeSet<ResolvedStateEvent>,
                               first_name: &str,
                               second_name: &str| {
        if first_resolved_state != second_resolved_state {
            let diff = unified_diff(
                Algorithm::default(),
                &to_json_string_pretty(first_resolved_state)
                    .expect("should be able to serialize first resolved state"),
                &to_json_string_pretty(second_resolved_state)
                    .expect("should be able to serialize second resolved state"),
                3,
                Some((first_name, second_name)),
            );

            panic!("{first_name} and {second_name} results should match; but they differ:\n{diff}");
        }
    };

    assert_states_match(
        &iteratively_resolved_state,
        &batched_resolved_state,
        "iterative",
        "batched",
    );
    assert_states_match(&batched_resolved_state, &atomic_resolved_state, "batched", "atomic");

    Snapshots { resolved_state: iteratively_resolved_state }
}

/// Test a list of JSON files containing a list of PDUs and a list of JSON files containing the
/// event IDs that form a contrived state and return the results.
fn test_contrived_states(pdus_paths: &[&str], state_sets_paths: &[&str]) -> Snapshots {
    let (pdu_batches, auth_rules, state_res_rules) = snapshot_test_prelude(pdus_paths);

    let pdus = pdu_batches.into_iter().flat_map(|x| x.into_iter()).collect::<Vec<_>>();

    let pdus_by_id: HashMap<EventId, Pdu> =
        pdus.clone().into_iter().map(|pdu| (pdu.event_id().to_owned(), pdu.to_owned())).collect();

    let state_sets = state_sets_paths
        .iter()
        .map(|x| {
            from_json_str::<Vec<EventId>>(
                &fs::read_to_string(FIXTURES_PATH.join(x))
                    .expect("should be able to read JSON file of event IDs"),
            )
            .expect("should be able to deserialize JSON file of event IDs")
            .into_iter()
            .map(|event_id| {
                pdus_by_id
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
                    .expect("Event IDs in JSON file must be in PDUs JSON")
            })
            .collect()
        })
        .collect::<Vec<StateMap<EventId>>>();

    let mut auth_chain_sets = Vec::new();
    for state_map in &state_sets {
        let mut auth_chain = HashSet::new();

        for event_id in state_map.values() {
            let pdu = pdus_by_id
                .get(event_id)
                .expect("We already confirmed all state set event ids have pdus");

            auth_chain.extend(
                auth_events_dfs(&pdus_by_id, pdu).expect("Auth events DFS should not fail"),
            );
        }

        auth_chain_sets.push(auth_chain);
    }

    let resolved_state = resolve(
        &auth_rules,
        &state_res_rules,
        &state_sets,
        auth_chain_sets,
        |x| pdus_by_id.get(x).cloned(),
        |conflicted_state_set| conflicted_state_subgraph_dfs(conflicted_state_set, &pdus_by_id),
    )
    .expect("atomic state resolution should succeed");

    Snapshots {
        resolved_state: reshape(&pdus_by_id, resolved_state)
            .expect("should be able to reshape atomic resolved state"),
    }
}

/// Perform state resolution on a batch of PDUs.
///
/// This function can be used to resolve the state of a room in a single call if all PDUs are
/// provided at once, or across multiple calls if given PDUs in batches in a loop. The latter form
/// simulates the case commonly experienced by homeservers during normal operation.
///
/// # Arguments
///
/// * `auth_rules`: The authorization rules of the room version.
/// * `state_res_rules`: The state resolution rules of the room version.
/// * `pdus`: An iterator of [`Pdu`]s to resolve, either alone or against the `prev_state`.
/// * `pdus_by_id`: A map of [`EventId`]s to the [`Pdu`] with that ID.
///   * Should be empty for the first call.
///   * Should not be mutated outside of this function.
/// * `prev_state`: The state returned by a previous call to this function, if any.
///   * Should be `None` for the first call.
///   * Should not be mutated outside of this function.
fn resolve_batch<'a, I, II>(
    auth_rules: &AuthorizationRules,
    state_res_rules: &StateResolutionV2Rules,
    pdus: II,
    pdus_by_id: &mut HashMap<EventId, Pdu>,
    prev_state: &mut Option<StateMap<EventId>>,
) -> Result<StateMap<EventId>, Box<dyn Error>>
where
    I: Iterator<Item = &'a Pdu>,
    II: IntoIterator<IntoIter = I> + Clone,
{
    let mut state_sets = prev_state.take().map(|x| vec![x]).unwrap_or_default();

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

        state_sets.push(state_map);
    }

    pdus_by_id
        .extend(pdus.clone().into_iter().map(|pdu| (pdu.event_id().to_owned(), pdu.to_owned())));

    let mut auth_chain_sets = Vec::new();
    for pdu in pdus {
        auth_chain_sets.push(auth_events_dfs(&*pdus_by_id, pdu)?);
    }

    resolve(
        auth_rules,
        state_res_rules,
        &state_sets,
        auth_chain_sets,
        |x| pdus_by_id.get(x).cloned(),
        |conflicted_state_set| conflicted_state_subgraph_dfs(conflicted_state_set, pdus_by_id),
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
/// * `auth_rules`: The authorization rules of the room version.
/// * `state_res_rules`: The state resolution rules of the room version.
/// * `pdus`: An iterator of [`Pdu`]s to resolve, with the following assumptions:
///   * `prev_events` of each PDU points to another provided state event.
///
/// # Returns
///
/// The state resolved by resolving all the leaves (PDUs which don't have any other PDUs pointing
/// to it via `prev_events`).
fn resolve_iteratively<'a, I, II>(
    auth_rules: &AuthorizationRules,
    state_res_rules: &StateResolutionV2Rules,
    pdus: II,
) -> Result<StateMap<EventId>, Box<dyn Error>>
where
    I: Iterator<Item = &'a Pdu>,
    II: IntoIterator<IntoIter = I> + Clone,
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

    let pdus_by_id: HashMap<EventId, Pdu> =
        HashMap::from_iter(pdus.into_iter().map(|pdu| (pdu.event_id().to_owned(), pdu.to_owned())));

    let auth_chain_from_state_map = |state_map: &StateMap<EventId>| -> Result<_, Box<dyn Error>> {
        let mut auth_chain_sets = HashSet::new();

        for event_id in state_map.values() {
            let pdu = pdus_by_id.get(event_id).expect("every pdu should be available");
            auth_chain_sets.extend(auth_events_dfs(&pdus_by_id, pdu)?);
        }

        Ok(auth_chain_sets)
    };

    let mut state_at_events: HashMap<EventId, StateMap<EventId>> = HashMap::new();
    let mut leaves = Vec::new();

    'outer: while let Some(event_id) = stack.pop() {
        let mut states_before_event = Vec::new();
        let mut auth_chains_before_event = Vec::new();

        let current_pdu = pdus_by_id.get(&event_id).expect("every pdu should be available");

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
            |x| pdus_by_id.get(x).cloned(),
            |conflicted_state_set| conflicted_state_subgraph_dfs(conflicted_state_set, &pdus_by_id),
        )?;

        let auth_chain_before_event = auth_chain_from_state_map(&state_before_event)?;

        let mut proposed_state_at_event = state_before_event.clone();
        proposed_state_at_event.insert(
            (
                current_pdu.event_type().to_string().into(),
                current_pdu.state_key().expect("all pdus are state events").to_owned(),
            ),
            event_id.clone(),
        );

        let mut auth_chain_at_event = auth_chain_before_event.clone();
        auth_chain_at_event.extend(auth_events_dfs(&pdus_by_id, current_pdu)?);

        let state_at_event = resolve(
            auth_rules,
            state_res_rules,
            &[state_before_event, proposed_state_at_event],
            vec![auth_chain_before_event, auth_chain_at_event],
            |x| pdus_by_id.get(x).cloned(),
            |conflicted_state_set| conflicted_state_subgraph_dfs(conflicted_state_set, &pdus_by_id),
        )?;

        state_at_events.insert(event_id.clone(), state_at_event);

        if let Some(prev_events) = forward_prev_events_graph.get(&event_id) {
            stack.extend(prev_events.iter().map(Deref::deref).cloned());
        } else {
            // pdu is a leaf: no `prev_events` point to it.
            leaves.push(event_id);
        }
    }

    if state_at_events.len() != pdus_by_id.len() {
        panic!(
            r#"Not all events have a state calculated!
                This is likely due to an event having a `prev_events`
                which points to a non-existent PDU."#
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
        |x| pdus_by_id.get(x).cloned(),
        |conflicted_state_set| conflicted_state_subgraph_dfs(conflicted_state_set, &pdus_by_id),
    )
    .map_err(Into::into)
}

/// Depth-first search for the `auth_events` of the given PDU.
///
/// # Errors
///
/// Fails if `pdus` does not contain a PDU that appears in the recursive `auth_events` of `pdu`.
fn auth_events_dfs(
    pdus_by_id: &HashMap<EventId, Pdu>,
    pdu: &Pdu,
) -> Result<HashSet<EventId>, Box<dyn Error>> {
    let mut out = HashSet::new();
    let mut stack = pdu.auth_events().cloned().collect::<Vec<_>>();

    while let Some(event_id) = stack.pop() {
        if out.contains(&event_id) {
            continue;
        }

        out.insert(event_id.clone());

        stack.extend(
            pdus_by_id
                .get(&event_id)
                .ok_or_else(|| format!("missing required PDU: {event_id}"))?
                .auth_events()
                .cloned(),
        );
    }

    Ok(out)
}

/// Retrieves the conflicted state subgraph, using Depth-first search on the `auth_events` of the
/// conflicted state events.
fn conflicted_state_subgraph_dfs(
    conflicted_state_set: &StateMap<Vec<EventId>>,
    pdus_by_id: &HashMap<EventId, Pdu>,
) -> Option<HashSet<EventId>> {
    let conflicted_event_ids: HashSet<_> =
        conflicted_state_set.values().flatten().cloned().collect();
    let mut conflicted_state_subgraph = HashSet::new();

    let mut stack = vec![conflicted_event_ids.iter().cloned().collect::<Vec<_>>()];
    let mut path = Vec::new();

    let mut seen_events = HashSet::new();

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

        stack.push(pdus_by_id.get(&event_id)?.auth_events().cloned().collect());

        seen_events.insert(event_id);
    }

    Some(conflicted_state_subgraph)
}
