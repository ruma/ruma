//! State resolution integration tests.

use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap, HashSet},
    error::Error,
    fs,
    path::Path,
};

use ruma_common::{
    room_version_rules::AuthorizationRules, MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId,
    OwnedUserId, RoomId, RoomVersionId, UserId,
};
use ruma_events::{StateEventType, TimelineEventType};
use ruma_state_res::{resolve, Event, StateMap};
use serde::{Deserialize, Serialize};
use serde_json::{
    from_str as from_json_str, to_string_pretty as to_json_string_pretty,
    to_value as to_json_value, value::RawValue as RawJsonValue, Error as JsonError,
    Value as JsonValue,
};
use similar::{udiff::unified_diff, Algorithm};
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter};

/// Create a new snapshot test.
///
/// # Arguments
///
/// * The test function's name.
/// * A list of JSON files relative to `tests/it/fixtures` to load PDUs to resolve from.
macro_rules! snapshot_test {
    ($name:ident, $paths:expr $(,)?) => {
        #[test]
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

// This module must be defined lexically after the `snapshot_test` macro.
mod snapshot_tests;

/// A persistent data unit.
#[derive(Deserialize, Clone)]
struct Pdu {
    event_id: OwnedEventId,
    room_id: OwnedRoomId,
    sender: OwnedUserId,
    origin_server_ts: MilliSecondsSinceUnixEpoch,
    #[serde(rename = "type")]
    kind: TimelineEventType,
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

    fn room_id(&self) -> &RoomId {
        &self.room_id
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
    event_id: OwnedEventId,

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

/// Test a list of JSON files containing a list of PDUs and return the results.
///
/// State resolution is run both atomically for all PDUs and in batches of PDUs by file.
fn test_resolve(paths: &[&str]) -> Snapshots {
    // Run `cargo test -- --show-output` to view traces, set `RUST_LOG` to control filtering.
    let _subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_test_writer()
        .finish()
        .set_default();

    let fixtures_path = Path::new("tests/it/fixtures");

    let pdu_batches = paths
        .iter()
        .map(|x| {
            from_json_str(
                &fs::read_to_string(fixtures_path.join(x))
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
    let rules = room_version_id.rules().expect("room version should be supported").authorization;

    // Resolve PDUs in batches by file
    let mut pdus_by_id = HashMap::new();
    let mut batched_resolved_state = None;
    for pdus in &pdu_batches {
        batched_resolved_state = Some(
            resolve_batch(&rules, pdus, &mut pdus_by_id, &mut batched_resolved_state)
                .expect("batched state resolution step should succeed"),
        );
    }
    let batched_resolved_state =
        batched_resolved_state.expect("batched state resolution should have run at least once");

    // Resolve all PDUs in a single step
    let atomic_resolved_state = resolve_batch(
        &rules,
        pdu_batches.iter().flat_map(|x| x.iter()),
        &mut HashMap::new(),
        &mut None,
    )
    .expect("atomic state resolution should succeed");

    // Reshape the data a bit to make the diff and snapshots easier to compare.
    let reshape = |x: StateMap<_>| {
        x.into_iter()
            .map(|((kind, state_key), event_id)| {
                Ok(ResolvedStateEvent {
                    kind,
                    state_key,
                    content: to_json_value(pdus_by_id[&event_id].content())?,
                    event_id,
                })
            })
            .collect::<Result<_, JsonError>>()
    };

    let batched_resolved_state =
        reshape(batched_resolved_state).expect("should be able to reshape batched resolved state");
    let atomic_resolved_state =
        reshape(atomic_resolved_state).expect("should be able to reshape atomic resolved state");

    if batched_resolved_state != atomic_resolved_state {
        let diff = unified_diff(
            Algorithm::default(),
            &to_json_string_pretty(&batched_resolved_state)
                .expect("should be able to serialize batched resolved state"),
            &to_json_string_pretty(&atomic_resolved_state)
                .expect("should be able to serialize atomic resolved state"),
            3,
            Some(("batched", "atomic")),
        );

        panic!("batched and atomic results should match; but they differ:\n{diff}");
    }

    Snapshots { resolved_state: batched_resolved_state }
}

/// Perform state resolution on a batch of PDUs.
///
/// This function can be used to resolve the state of a room in a single call if all PDUs are
/// provided at once, or across multiple calls if given PDUs in batches in a loop. The latter form
/// simulates the case commonly experienced by homeservers during normal operation.
///
/// # Arguments
///
/// * `rules`: The rules of the room version.
/// * `pdus`: An iterator of [`Pdu`]s to resolve, either alone or against the `prev_state`.
/// * `pdus_by_id`: A map of [`OwnedEventId`]s to the [`Pdu`] with that ID.
///   * Should be empty for the first call.
///   * Should not be mutated outside of this function.
/// * `prev_state`: The state returned by a previous call to this function, if any.
///   * Should be `None` for the first call.
///   * Should not be mutated outside of this function.
fn resolve_batch<'a, I, II>(
    rules: &AuthorizationRules,
    pdus: II,
    pdus_by_id: &mut HashMap<OwnedEventId, Pdu>,
    prev_state: &mut Option<StateMap<OwnedEventId>>,
) -> Result<StateMap<OwnedEventId>, Box<dyn Error>>
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

    resolve(rules, &state_sets, auth_chain_sets, |x| pdus_by_id.get(x).cloned()).map_err(Into::into)
}

/// Depth-first search for the `auth_events` of the given PDU.
///
/// # Errors
///
/// Fails if `pdus` does not contain a PDU that appears in the recursive `auth_events` of `pdu`.
fn auth_events_dfs(
    pdus_by_id: &HashMap<OwnedEventId, Pdu>,
    pdu: &Pdu,
) -> Result<HashSet<OwnedEventId>, Box<dyn Error>> {
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
                .ok_or_else(|| format!("missing required PDU: {}", event_id))?
                .auth_events()
                .cloned(),
        );
    }

    Ok(out)
}
