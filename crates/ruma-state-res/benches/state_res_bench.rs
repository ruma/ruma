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
    convert::TryInto,
    sync::{
        atomic::{AtomicU64, Ordering::SeqCst},
        Arc,
    },
};

use criterion::{criterion_group, criterion_main, Criterion};
use event::PduEvent;
use js_int::{int, uint};
use maplit::{btreemap, hashmap, hashset};
use ruma_common::{
    events::{
        pdu::{EventHash, Pdu, RoomV3Pdu},
        room::{
            join_rules::{JoinRule, RoomJoinRulesEventContent},
            member::{MembershipState, RoomMemberEventContent},
        },
        RoomEventType,
    },
    room_id, user_id, EventId, MilliSecondsSinceUnixEpoch, RoomId, RoomVersionId, UserId,
};
use ruma_state_res::{self as state_res, Error, Event, Result, StateMap};
use serde_json::{
    json,
    value::{to_raw_value as to_raw_json_value, RawValue as RawJsonValue},
};

static SERVER_TIMESTAMP: AtomicU64 = AtomicU64::new(0);

fn lexico_topo_sort(c: &mut Criterion) {
    c.bench_function("lexicographical topological sort", |b| {
        let graph = hashmap! {
            event_id("l") => hashset![event_id("o")],
            event_id("m") => hashset![event_id("n"), event_id("o")],
            event_id("n") => hashset![event_id("o")],
            event_id("o") => hashset![], // "o" has zero outgoing edges but 4 incoming edges
            event_id("p") => hashset![event_id("o")],
        };
        b.iter(|| {
            let _ = state_res::lexicographical_topological_sort(&graph, |_id| {
                Ok((int!(0), MilliSecondsSinceUnixEpoch(uint!(0))))
            });
        })
    });
}

fn resolution_shallow_auth_chain(c: &mut Criterion) {
    c.bench_function("resolve state of 5 events one fork", |b| {
        let mut store = TestStore(hashmap! {});

        // build up the DAG
        let (state_at_bob, state_at_charlie, _) = store.set_up();

        b.iter(|| {
            let ev_map = store.0.clone();
            let state_sets = [&state_at_bob, &state_at_charlie];
            let _ = match state_res::resolve(
                &RoomVersionId::V6,
                state_sets,
                state_sets
                    .iter()
                    .map(|map| {
                        store.auth_event_ids(room_id(), map.values().cloned().collect()).unwrap()
                    })
                    .collect(),
                |id| ev_map.get(id).map(Arc::clone),
            ) {
                Ok(state) => state,
                Err(e) => panic!("{}", e),
            };
        })
    });
}

fn resolve_deeper_event_set(c: &mut Criterion) {
    c.bench_function("resolve state of 10 events 3 conflicting", |b| {
        let mut inner = INITIAL_EVENTS();
        let ban = BAN_STATE_SET();

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
            (
                (ev.event_type().to_owned(), ev.state_key().unwrap().to_owned()),
                ev.event_id().to_owned(),
            )
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
            (
                (ev.event_type().to_owned(), ev.state_key().unwrap().to_owned()),
                ev.event_id().to_owned(),
            )
        })
        .collect::<StateMap<_>>();

        b.iter(|| {
            let state_sets = [&state_set_a, &state_set_b];
            let _ = match state_res::resolve(
                &RoomVersionId::V6,
                state_sets,
                state_sets
                    .iter()
                    .map(|map| {
                        store.auth_event_ids(room_id(), map.values().cloned().collect()).unwrap()
                    })
                    .collect(),
                |id| inner.get(id).map(Arc::clone),
            ) {
                Ok(state) => state,
                Err(_) => panic!("resolution failed during benchmarking"),
            };
        })
    });
}

criterion_group!(
    benches,
    lexico_topo_sort,
    resolution_shallow_auth_chain,
    resolve_deeper_event_set
);

criterion_main!(benches);

//*/////////////////////////////////////////////////////////////////////
//
//  IMPLEMENTATION DETAILS AHEAD
//
/////////////////////////////////////////////////////////////////////*/
struct TestStore<E: Event>(HashMap<Box<EventId>, Arc<E>>);

#[allow(unused)]
impl<E: Event> TestStore<E> {
    fn get_event(&self, room_id: &RoomId, event_id: &EventId) -> Result<Arc<E>> {
        self.0
            .get(event_id)
            .map(Arc::clone)
            .ok_or_else(|| Error::NotFound(format!("{} not found", event_id)))
    }

    /// Returns the events that correspond to the `event_ids` sorted in the same order.
    fn get_events(&self, room_id: &RoomId, event_ids: &[Box<EventId>]) -> Result<Vec<Arc<E>>> {
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
        while !stack.is_empty() {
            let ev_id = stack.pop().unwrap();
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
    fn set_up(
        &mut self,
    ) -> (StateMap<Box<EventId>>, StateMap<Box<EventId>>, StateMap<Box<EventId>>) {
        let create_event = to_pdu_event::<&EventId>(
            "CREATE",
            alice(),
            RoomEventType::RoomCreate,
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
            RoomEventType::RoomMember,
            Some(alice().to_string().as_str()),
            member_content_join(),
            &[cre.clone()],
            &[cre.clone()],
        );
        self.0.insert(alice_mem.event_id().to_owned(), Arc::clone(&alice_mem));

        let join_rules = to_pdu_event(
            "IJR",
            alice(),
            RoomEventType::RoomJoinRules,
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
            RoomEventType::RoomMember,
            Some(bob().to_string().as_str()),
            member_content_join(),
            &[cre.clone(), join_rules.event_id().to_owned()],
            &[join_rules.event_id().to_owned()],
        );
        self.0.insert(bob_mem.event_id().to_owned(), bob_mem.clone());

        let charlie_mem = to_pdu_event(
            "IMC",
            charlie(),
            RoomEventType::RoomMember,
            Some(charlie().to_string().as_str()),
            member_content_join(),
            &[cre, join_rules.event_id().to_owned()],
            &[join_rules.event_id().to_owned()],
        );
        self.0.insert(charlie_mem.event_id().to_owned(), charlie_mem.clone());

        let state_at_bob = [&create_event, &alice_mem, &join_rules, &bob_mem]
            .iter()
            .map(|e| {
                (
                    (e.event_type().to_owned(), e.state_key().unwrap().to_owned()),
                    e.event_id().to_owned(),
                )
            })
            .collect::<StateMap<_>>();

        let state_at_charlie = [&create_event, &alice_mem, &join_rules, &charlie_mem]
            .iter()
            .map(|e| {
                (
                    (e.event_type().to_owned(), e.state_key().unwrap().to_owned()),
                    e.event_id().to_owned(),
                )
            })
            .collect::<StateMap<_>>();

        let expected = [&create_event, &alice_mem, &join_rules, &bob_mem, &charlie_mem]
            .iter()
            .map(|e| {
                (
                    (e.event_type().to_owned(), e.state_key().unwrap().to_owned()),
                    e.event_id().to_owned(),
                )
            })
            .collect::<StateMap<_>>();

        (state_at_bob, state_at_charlie, expected)
    }
}

fn event_id(id: &str) -> Box<EventId> {
    if id.contains('$') {
        return id.try_into().unwrap();
    }
    format!("${}:foo", id).try_into().unwrap()
}

fn alice() -> Box<UserId> {
    user_id!("@alice:foo").to_owned()
}

fn bob() -> Box<UserId> {
    user_id!("@bob:foo").to_owned()
}

fn charlie() -> Box<UserId> {
    user_id!("@charlie:foo").to_owned()
}

fn ella() -> Box<UserId> {
    user_id!("@ella:foo").to_owned()
}

fn room_id() -> &'static RoomId {
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
    sender: Box<UserId>,
    ev_type: RoomEventType,
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
    let id = if id.contains('$') { id.to_owned() } else { format!("${}:foo", id) };
    let auth_events = auth_events.iter().map(AsRef::as_ref).map(event_id).collect::<Vec<_>>();
    let prev_events = prev_events.iter().map(AsRef::as_ref).map(event_id).collect::<Vec<_>>();

    let state_key = state_key.map(ToOwned::to_owned);
    Arc::new(PduEvent {
        event_id: id.try_into().unwrap(),
        rest: Pdu::RoomV3Pdu(RoomV3Pdu {
            room_id: room_id().to_owned(),
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
            hashes: EventHash::new(String::new()),
            signatures: btreemap! {},
        }),
    })
}

// all graphs start with these input events
#[allow(non_snake_case)]
fn INITIAL_EVENTS() -> HashMap<Box<EventId>, Arc<PduEvent>> {
    vec![
        to_pdu_event::<&EventId>(
            "CREATE",
            alice(),
            RoomEventType::RoomCreate,
            Some(""),
            to_raw_json_value(&json!({ "creator": alice() })).unwrap(),
            &[],
            &[],
        ),
        to_pdu_event(
            "IMA",
            alice(),
            RoomEventType::RoomMember,
            Some(alice().to_string().as_str()),
            member_content_join(),
            &["CREATE"],
            &["CREATE"],
        ),
        to_pdu_event(
            "IPOWER",
            alice(),
            RoomEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice().to_string(): 100 } })).unwrap(),
            &["CREATE", "IMA"],
            &["IMA"],
        ),
        to_pdu_event(
            "IJR",
            alice(),
            RoomEventType::RoomJoinRules,
            Some(""),
            to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Public)).unwrap(),
            &["CREATE", "IMA", "IPOWER"],
            &["IPOWER"],
        ),
        to_pdu_event(
            "IMB",
            bob(),
            RoomEventType::RoomMember,
            Some(bob().to_string().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "IPOWER"],
            &["IJR"],
        ),
        to_pdu_event(
            "IMC",
            charlie(),
            RoomEventType::RoomMember,
            Some(charlie().to_string().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "IPOWER"],
            &["IMB"],
        ),
        to_pdu_event::<&EventId>(
            "START",
            charlie(),
            RoomEventType::RoomTopic,
            Some(""),
            to_raw_json_value(&json!({})).unwrap(),
            &[],
            &[],
        ),
        to_pdu_event::<&EventId>(
            "END",
            charlie(),
            RoomEventType::RoomTopic,
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
fn BAN_STATE_SET() -> HashMap<Box<EventId>, Arc<PduEvent>> {
    vec![
        to_pdu_event(
            "PA",
            alice(),
            RoomEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice(): 100, bob(): 50 } })).unwrap(),
            &["CREATE", "IMA", "IPOWER"], // auth_events
            &["START"],                   // prev_events
        ),
        to_pdu_event(
            "PB",
            alice(),
            RoomEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({ "users": { alice(): 100, bob(): 50 } })).unwrap(),
            &["CREATE", "IMA", "IPOWER"],
            &["END"],
        ),
        to_pdu_event(
            "MB",
            alice(),
            RoomEventType::RoomMember,
            Some(ella().as_str()),
            member_content_ban(),
            &["CREATE", "IMA", "PB"],
            &["PA"],
        ),
        to_pdu_event(
            "IME",
            ella(),
            RoomEventType::RoomMember,
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

mod event {
    use ruma_common::{
        events::{pdu::Pdu, RoomEventType},
        EventId, MilliSecondsSinceUnixEpoch, RoomId, UserId,
    };
    use ruma_state_res::Event;
    use serde::{Deserialize, Serialize};
    use serde_json::value::RawValue as RawJsonValue;

    impl Event for PduEvent {
        type Id = Box<EventId>;

        fn event_id(&self) -> &Self::Id {
            &self.event_id
        }

        fn room_id(&self) -> &RoomId {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => &ev.room_id,
                Pdu::RoomV3Pdu(ev) => &ev.room_id,
                #[cfg(not(feature = "unstable-exhaustive-types"))]
                _ => unreachable!("new PDU version"),
            }
        }

        fn sender(&self) -> &UserId {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => &ev.sender,
                Pdu::RoomV3Pdu(ev) => &ev.sender,
                #[cfg(not(feature = "unstable-exhaustive-types"))]
                _ => unreachable!("new PDU version"),
            }
        }

        fn event_type(&self) -> &RoomEventType {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => &ev.kind,
                Pdu::RoomV3Pdu(ev) => &ev.kind,
                #[cfg(not(feature = "unstable-exhaustive-types"))]
                _ => unreachable!("new PDU version"),
            }
        }

        fn content(&self) -> &RawJsonValue {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => &ev.content,
                Pdu::RoomV3Pdu(ev) => &ev.content,
                #[cfg(not(feature = "unstable-exhaustive-types"))]
                _ => unreachable!("new PDU version"),
            }
        }

        fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => ev.origin_server_ts,
                Pdu::RoomV3Pdu(ev) => ev.origin_server_ts,
                #[cfg(not(feature = "unstable-exhaustive-types"))]
                _ => unreachable!("new PDU version"),
            }
        }

        fn state_key(&self) -> Option<&str> {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => ev.state_key.as_deref(),
                Pdu::RoomV3Pdu(ev) => ev.state_key.as_deref(),
                #[cfg(not(feature = "unstable-exhaustive-types"))]
                _ => unreachable!("new PDU version"),
            }
        }

        fn prev_events(&self) -> Box<dyn DoubleEndedIterator<Item = &Self::Id> + '_> {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => Box::new(ev.prev_events.iter().map(|(id, _)| id)),
                Pdu::RoomV3Pdu(ev) => Box::new(ev.prev_events.iter()),
                #[cfg(not(feature = "unstable-exhaustive-types"))]
                _ => unreachable!("new PDU version"),
            }
        }

        fn auth_events(&self) -> Box<dyn DoubleEndedIterator<Item = &Self::Id> + '_> {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => Box::new(ev.auth_events.iter().map(|(id, _)| id)),
                Pdu::RoomV3Pdu(ev) => Box::new(ev.auth_events.iter()),
                #[cfg(not(feature = "unstable-exhaustive-types"))]
                _ => unreachable!("new PDU version"),
            }
        }

        fn redacts(&self) -> Option<&Self::Id> {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => ev.redacts.as_ref(),
                Pdu::RoomV3Pdu(ev) => ev.redacts.as_ref(),
                #[cfg(not(feature = "unstable-exhaustive-types"))]
                _ => unreachable!("new PDU version"),
            }
        }
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PduEvent {
        pub event_id: Box<EventId>,
        #[serde(flatten)]
        pub rest: Pdu,
    }
}
