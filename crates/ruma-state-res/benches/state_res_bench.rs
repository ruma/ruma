// Because of criterion `cargo bench` works,
// but if you use `cargo bench -- --save-baseline <name>`
// or pass any other args to it, it fails with the error
// `cargo bench unknown option --save-baseline`.
// To pass args to criterion, use this form
// `cargo bench --bench <name of the bench> -- --save-baseline <name>`.

#![allow(clippy::exhaustive_structs)]

use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    slice,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering::SeqCst},
    },
};

use criterion::{Criterion, criterion_group, criterion_main};
use event::{EventHash, PduEvent};
use js_int::{int, uint};
use maplit::{btreemap, hashmap, hashset};
use ruma_common::{
    EventId, MilliSecondsSinceUnixEpoch, RoomId, UserId, room_id,
    room_version_rules::{AuthorizationRules, StateResolutionV2Rules},
    user_id,
};
use ruma_events::{
    StateEventType, TimelineEventType,
    room::{
        join_rules::{JoinRule, RoomJoinRulesEventContent},
        member::{MembershipState, RoomMemberEventContent},
    },
};
use ruma_state_res::{self as state_res, Error, Event, Result, StateMap};
use serde_json::{
    json,
    value::{RawValue as RawJsonValue, to_raw_value as to_raw_json_value},
};

static SERVER_TIMESTAMP: AtomicU64 = AtomicU64::new(0);

fn reverse_topological_power_sort(c: &mut Criterion) {
    c.bench_function("reverse_topological_power_sort", |b| {
        let graph = hashmap! {
            event_id("l") => hashset![event_id("o")],
            event_id("m") => hashset![event_id("n"), event_id("o")],
            event_id("n") => hashset![event_id("o")],
            event_id("o") => hashset![], // "o" has zero outgoing edges but 4 incoming edges
            event_id("p") => hashset![event_id("o")],
        };
        b.iter(|| {
            let _ = state_res::reverse_topological_power_sort(&graph, |_id| {
                Ok((int!(0).into(), MilliSecondsSinceUnixEpoch(uint!(0))))
            });
        });
    });
}

fn resolution_shallow_auth_chain(c: &mut Criterion) {
    c.bench_function("resolve state of 5 events one fork", |b| {
        let room_id = room_id();
        let mut store = TestStore(hashmap! {});

        // build up the DAG
        let (state_at_bob, state_at_charlie, _) = store.set_up();

        b.iter(|| {
            let ev_map = store.0.clone();
            let state_sets = [&state_at_bob, &state_at_charlie];
            let _ = match state_res::resolve(
                &AuthorizationRules::V6,
                &StateResolutionV2Rules::V2_0,
                state_sets,
                state_sets
                    .iter()
                    .map(|map| {
                        store.auth_event_ids(&room_id, map.values().cloned().collect()).unwrap()
                    })
                    .collect(),
                |id| ev_map.get(id).map(Arc::clone),
                |_| unreachable!(),
            ) {
                Ok(state) => state,
                Err(e) => panic!("{e}"),
            };
        });
    });
}

fn resolve_deeper_event_set(c: &mut Criterion) {
    c.bench_function("resolve state of 10 events 3 conflicting", |b| {
        let mut inner = INITIAL_EVENTS();
        let ban = BAN_STATE_SET();
        let room_id = room_id();

        inner.extend(ban);
        let store = TestStore(inner.clone());

        let state_set_a = [
            inner.get(&event_id("CREATE")).unwrap(),
            inner.get(&event_id("IJR")).unwrap(),
            inner.get(&event_id("IMA")).unwrap(),
            inner.get(&event_id("IMB")).unwrap(),
            inner.get(&event_id("IMC")).unwrap(),
            inner.get(&event_id("MB")).unwrap(),
            inner.get(&event_id("PA")).unwrap(),
        ]
        .iter()
        .map(|ev| {
            (ev.event_type().with_state_key(ev.state_key().unwrap()), ev.event_id().to_owned())
        })
        .collect::<StateMap<_>>();

        let state_set_b = [
            inner.get(&event_id("CREATE")).unwrap(),
            inner.get(&event_id("IJR")).unwrap(),
            inner.get(&event_id("IMA")).unwrap(),
            inner.get(&event_id("IMB")).unwrap(),
            inner.get(&event_id("IMC")).unwrap(),
            inner.get(&event_id("IME")).unwrap(),
            inner.get(&event_id("PA")).unwrap(),
        ]
        .iter()
        .map(|ev| {
            (ev.event_type().with_state_key(ev.state_key().unwrap()), ev.event_id().to_owned())
        })
        .collect::<StateMap<_>>();

        b.iter(|| {
            let state_sets = [&state_set_a, &state_set_b];
            state_res::resolve(
                &AuthorizationRules::V6,
                &StateResolutionV2Rules::V2_0,
                state_sets,
                state_sets
                    .iter()
                    .map(|map| {
                        store.auth_event_ids(&room_id, map.values().cloned().collect()).unwrap()
                    })
                    .collect(),
                |id| inner.get(id).map(Arc::clone),
                |_| unreachable!(),
            )
            .unwrap_or_else(|_| panic!("resolution failed during benchmarking"));
        });
    });
}

criterion_group!(
    benches,
    reverse_topological_power_sort,
    resolution_shallow_auth_chain,
    resolve_deeper_event_set
);

criterion_main!(benches);

//*/////////////////////////////////////////////////////////////////////
//
//  IMPLEMENTATION DETAILS AHEAD
//
/////////////////////////////////////////////////////////////////////*/
struct TestStore<E: Event>(HashMap<EventId, Arc<E>>);

#[allow(unused)]
impl<E: Event> TestStore<E> {
    fn get_event(&self, room_id: &RoomId, event_id: &EventId) -> Result<Arc<E>> {
        self.0.get(event_id).map(Arc::clone).ok_or_else(|| Error::NotFound(event_id.to_owned()))
    }

    /// Returns the events that correspond to the `event_ids` sorted in the same order.
    fn get_events(&self, room_id: &RoomId, event_ids: &[EventId]) -> Result<Vec<Arc<E>>> {
        let mut events = vec![];
        for id in event_ids {
            events.push(self.get_event(room_id, id)?);
        }
        Ok(events)
    }

    /// Returns a Vec of the related auth events to the given `event`.
    fn auth_event_ids(&self, room_id: &RoomId, event_ids: Vec<E::Id>) -> Result<HashSet<E::Id>> {
        let mut result = HashSet::new();
        let mut stack = event_ids;

        // DFS for auth event chain
        while let Some(ev_id) = stack.pop() {
            if result.contains(&ev_id) {
                continue;
            }

            result.insert(ev_id.clone());

            let event = self.get_event(room_id, ev_id.borrow())?;

            stack.extend(event.auth_events().map(ToOwned::to_owned));
        }

        Ok(result)
    }

    /// Returns a vector representing the difference in auth chains of the given `events`.
    fn auth_chain_diff(&self, room_id: &RoomId, event_ids: Vec<Vec<E::Id>>) -> Result<Vec<E::Id>> {
        let mut auth_chain_sets = vec![];
        for ids in event_ids {
            // TODO state store `auth_event_ids` returns self in the event ids list
            // when an event returns `auth_event_ids` self is not contained
            let chain = self.auth_event_ids(room_id, ids)?.into_iter().collect::<HashSet<_>>();
            auth_chain_sets.push(chain);
        }

        if let Some(first) = auth_chain_sets.first().cloned() {
            let common = auth_chain_sets
                .iter()
                .skip(1)
                .fold(first, |a, b| a.intersection(b).cloned().collect::<HashSet<_>>());

            Ok(auth_chain_sets
                .into_iter()
                .flatten()
                .filter(|id| !common.contains(id.borrow()))
                .collect())
        } else {
            Ok(vec![])
        }
    }
}

impl TestStore<PduEvent> {
    #[allow(clippy::type_complexity)]
    fn set_up(&mut self) -> (StateMap<EventId>, StateMap<EventId>, StateMap<EventId>) {
        let create_event = to_pdu_event::<&EventId>(
            "CREATE",
            alice(),
            TimelineEventType::RoomCreate,
            Some(""),
            to_raw_json_value(&json!({ "creator": alice() })).unwrap(),
            &[],
            &[],
        );
        let cre = create_event.event_id().to_owned();
        self.0.insert(cre.clone(), Arc::clone(&create_event));

        let alice_mem = to_pdu_event(
            "IMA",
            alice(),
            TimelineEventType::RoomMember,
            Some(alice().to_string().as_str()),
            member_content_join(),
            slice::from_ref(&cre),
            slice::from_ref(&cre),
        );
        self.0.insert(alice_mem.event_id().to_owned(), Arc::clone(&alice_mem));

        let join_rules = to_pdu_event(
            "IJR",
            alice(),
            TimelineEventType::RoomJoinRules,
            Some(""),
            to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Public)).unwrap(),
            &[cre.clone(), alice_mem.event_id().to_owned()],
            &[alice_mem.event_id().to_owned()],
        );
        self.0.insert(join_rules.event_id().to_owned(), join_rules.clone());

        // Bob and Charlie join at the same time, so there is a fork
        // this will be represented in the state_sets when we resolve
        let bob_mem = to_pdu_event(
            "IMB",
            bob(),
            TimelineEventType::RoomMember,
            Some(bob().to_string().as_str()),
            member_content_join(),
            &[cre.clone(), join_rules.event_id().to_owned()],
            &[join_rules.event_id().to_owned()],
        );
        self.0.insert(bob_mem.event_id().to_owned(), bob_mem.clone());

        let charlie_mem = to_pdu_event(
            "IMC",
            charlie(),
            TimelineEventType::RoomMember,
            Some(charlie().to_string().as_str()),
            member_content_join(),
            &[cre, join_rules.event_id().to_owned()],
            &[join_rules.event_id().to_owned()],
        );
        self.0.insert(charlie_mem.event_id().to_owned(), charlie_mem.clone());

        let state_at_bob = [&create_event, &alice_mem, &join_rules, &bob_mem]
            .iter()
            .map(|e| {
                (e.event_type().with_state_key(e.state_key().unwrap()), e.event_id().to_owned())
            })
            .collect::<StateMap<_>>();

        let state_at_charlie = [&create_event, &alice_mem, &join_rules, &charlie_mem]
            .iter()
            .map(|e| {
                (e.event_type().with_state_key(e.state_key().unwrap()), e.event_id().to_owned())
            })
            .collect::<StateMap<_>>();

        let expected = [&create_event, &alice_mem, &join_rules, &bob_mem, &charlie_mem]
            .iter()
            .map(|e| {
                (e.event_type().with_state_key(e.state_key().unwrap()), e.event_id().to_owned())
            })
            .collect::<StateMap<_>>();

        (state_at_bob, state_at_charlie, expected)
    }
}

fn event_id(id: &str) -> EventId {
    if id.contains('$') {
        return id.try_into().unwrap();
    }
    format!("${id}:foo").try_into().unwrap()
}

fn alice() -> UserId {
    user_id!("@alice:foo")
}

fn bob() -> UserId {
    user_id!("@bob:foo")
}

fn charlie() -> UserId {
    user_id!("@charlie:foo")
}

fn ella() -> UserId {
    user_id!("@ella:foo")
}

fn room_id() -> RoomId {
    room_id!("!test:foo")
}

fn member_content_ban() -> Box<RawJsonValue> {
    to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Ban)).unwrap()
}

fn member_content_join() -> Box<RawJsonValue> {
    to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Join)).unwrap()
}

fn to_pdu_event<S>(
    id: &str,
    sender: UserId,
    ev_type: TimelineEventType,
    state_key: Option<&str>,
    content: Box<RawJsonValue>,
    auth_events: &[S],
    prev_events: &[S],
) -> Arc<PduEvent>
where
    S: AsRef<str>,
{
    // We don't care if the addition happens in order just that it is atomic
    // (each event has its own value)
    let ts = SERVER_TIMESTAMP.fetch_add(1, SeqCst);
    let id = if id.contains('$') { id.to_owned() } else { format!("${id}:foo") };
    let auth_events = auth_events.iter().map(AsRef::as_ref).map(event_id).collect::<Vec<_>>();
    let prev_events = prev_events.iter().map(AsRef::as_ref).map(event_id).collect::<Vec<_>>();

    let state_key = state_key.map(ToOwned::to_owned);
    Arc::new(PduEvent {
        event_id: id.try_into().unwrap(),
        room_id: Some(room_id()),
        sender,
        origin_server_ts: MilliSecondsSinceUnixEpoch(ts.try_into().unwrap()),
        state_key,
        kind: ev_type,
        content,
        redacts: None,
        unsigned: btreemap! {},
        auth_events,
        prev_events,
        depth: uint!(0),
        hashes: EventHash { sha256: "".to_owned() },
        signatures: Default::default(),
        rejected: false,
    })
}

// all graphs start with these input events
#[allow(non_snake_case)]
fn INITIAL_EVENTS() -> HashMap<EventId, Arc<PduEvent>> {
    vec![
        to_pdu_event::<&EventId>(
            "CREATE",
            alice(),
            TimelineEventType::RoomCreate,
            Some(""),
            to_raw_json_value(&json!({ "creator": alice() })).unwrap(),
            &[],
            &[],
        ),
        to_pdu_event(
            "IMA",
            alice(),
            TimelineEventType::RoomMember,
            Some(alice().as_str()),
            member_content_join(),
            &["CREATE"],
            &["CREATE"],
        ),
        to_pdu_event(
            "IPOWER",
            alice(),
            TimelineEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice(): 100 } })).unwrap(),
            &["CREATE", "IMA"],
            &["IMA"],
        ),
        to_pdu_event(
            "IJR",
            alice(),
            TimelineEventType::RoomJoinRules,
            Some(""),
            to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Public)).unwrap(),
            &["CREATE", "IMA", "IPOWER"],
            &["IPOWER"],
        ),
        to_pdu_event(
            "IMB",
            bob(),
            TimelineEventType::RoomMember,
            Some(bob().to_string().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "IPOWER"],
            &["IJR"],
        ),
        to_pdu_event(
            "IMC",
            charlie(),
            TimelineEventType::RoomMember,
            Some(charlie().to_string().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "IPOWER"],
            &["IMB"],
        ),
        to_pdu_event::<&EventId>(
            "START",
            charlie(),
            TimelineEventType::RoomTopic,
            Some(""),
            to_raw_json_value(&json!({})).unwrap(),
            &[],
            &[],
        ),
        to_pdu_event::<&EventId>(
            "END",
            charlie(),
            TimelineEventType::RoomTopic,
            Some(""),
            to_raw_json_value(&json!({})).unwrap(),
            &[],
            &[],
        ),
    ]
    .into_iter()
    .map(|ev| (ev.event_id().to_owned(), ev))
    .collect()
}

// all graphs start with these input events
#[allow(non_snake_case)]
fn BAN_STATE_SET() -> HashMap<EventId, Arc<PduEvent>> {
    vec![
        to_pdu_event(
            "PA",
            alice(),
            TimelineEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice(): 100, bob(): 50 } })).unwrap(),
            &["CREATE", "IMA", "IPOWER"], // auth_events
            &["START"],                   // prev_events
        ),
        to_pdu_event(
            "PB",
            alice(),
            TimelineEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice(): 100, bob(): 50 } })).unwrap(),
            &["CREATE", "IMA", "IPOWER"],
            &["END"],
        ),
        to_pdu_event(
            "MB",
            alice(),
            TimelineEventType::RoomMember,
            Some(ella().as_str()),
            member_content_ban(),
            &["CREATE", "IMA", "PB"],
            &["PA"],
        ),
        to_pdu_event(
            "IME",
            ella(),
            TimelineEventType::RoomMember,
            Some(ella().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "PA"],
            &["MB"],
        ),
    ]
    .into_iter()
    .map(|ev| (ev.event_id().to_owned(), ev))
    .collect()
}

/// Convenience trait for adding event type plus state key to state maps.
trait EventTypeExt {
    fn with_state_key(self, state_key: impl Into<String>) -> (StateEventType, String);
}

impl EventTypeExt for &TimelineEventType {
    fn with_state_key(self, state_key: impl Into<String>) -> (StateEventType, String) {
        (self.to_string().into(), state_key.into())
    }
}

pub(crate) mod event {
    use std::collections::BTreeMap;

    use js_int::UInt;
    use ruma_common::{EventId, MilliSecondsSinceUnixEpoch, RoomId, ServerSignatures, UserId};
    use ruma_events::TimelineEventType;
    use serde::{Deserialize, Serialize};
    use serde_json::value::RawValue as RawJsonValue;

    use crate::Event;

    impl Event for PduEvent {
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

        fn event_type(&self) -> &TimelineEventType {
            &self.kind
        }

        fn content(&self) -> &RawJsonValue {
            &self.content
        }

        fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch {
            self.origin_server_ts
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

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[allow(clippy::exhaustive_structs)]
    pub(crate) struct PduEvent {
        /// The ID of the event.
        pub(crate) event_id: EventId,

        /// The room this event belongs to.
        pub(crate) room_id: Option<RoomId>,

        /// The user id of the user who sent this event.
        pub(crate) sender: UserId,

        /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver
        /// of when this event was created.
        pub(crate) origin_server_ts: MilliSecondsSinceUnixEpoch,

        /// The event's type.
        #[serde(rename = "type")]
        pub(crate) kind: TimelineEventType,

        /// The event's content.
        pub(crate) content: Box<RawJsonValue>,

        /// A key that determines which piece of room state the event represents.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) state_key: Option<String>,

        /// Event IDs for the most recent events in the room that the homeserver was
        /// aware of when it created this event.
        pub(crate) prev_events: Vec<EventId>,

        /// The maximum depth of the `prev_events`, plus one.
        pub(crate) depth: UInt,

        /// Event IDs for the authorization events that would allow this event to be
        /// in the room.
        pub(crate) auth_events: Vec<EventId>,

        /// For redaction events, the ID of the event being redacted.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) redacts: Option<EventId>,

        /// Additional data added by the origin server but not covered by the
        /// signatures.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub(crate) unsigned: BTreeMap<String, Box<RawJsonValue>>,

        /// Content hashes of the PDU.
        pub(crate) hashes: EventHash,

        /// Signatures for the PDU.
        pub(crate) signatures: ServerSignatures,

        /// Whether the PDU was rejected.
        pub(crate) rejected: bool,
    }

    /// Content hashes of a PDU.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[allow(clippy::exhaustive_structs)]
    pub(crate) struct EventHash {
        /// The SHA-256 hash.
        pub(crate) sha256: String,
    }
}
