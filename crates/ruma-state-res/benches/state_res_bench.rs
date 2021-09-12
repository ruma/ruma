// Because of criterion `cargo bench` works,
// but if you use `cargo bench -- --save-baseline <name>`
// or pass any other args to it, it fails with the error
// `cargo bench unknown option --save-baseline`.
// To pass args to criterion, use this form
// `cargo bench --bench <name of the bench> -- --save-baseline <name>`.

#![allow(clippy::exhaustive_structs)]

use std::{
    collections::{HashMap, HashSet},
    convert::{TryFrom, TryInto},
    sync::{
        atomic::{AtomicU64, Ordering::SeqCst},
        Arc,
    },
};

use criterion::{criterion_group, criterion_main, Criterion};
use event::StateEvent;
use js_int::uint;
use maplit::{btreemap, hashmap, hashset};
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events::{
    pdu::{EventHash, Pdu, RoomV3Pdu},
    room::{
        join_rules::JoinRule,
        member::{MemberEventContent, MembershipState},
    },
    EventType,
};
use ruma_identifiers::{EventId, RoomId, RoomVersionId, UserId};
use ruma_state_res::{self as state_res, Error, Event, Result, StateMap};
use serde_json::{json, Value as JsonValue};

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
            let _ = state_res::lexicographical_topological_sort(&graph, |id| {
                Ok((0, MilliSecondsSinceUnixEpoch(uint!(0)), id.clone()))
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
            let state_sets = vec![state_at_bob.clone(), state_at_charlie.clone()];
            let _ = match state_res::resolve(
                &RoomVersionId::Version6,
                &state_sets,
                state_sets
                    .iter()
                    .map(|map| {
                        store
                            .auth_event_ids(&room_id(), &map.values().cloned().collect::<Vec<_>>())
                            .unwrap()
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
                ev.event_id().clone(),
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
                ev.event_id().clone(),
            )
        })
        .collect::<StateMap<_>>();

        b.iter(|| {
            let state_sets = vec![state_set_a.clone(), state_set_b.clone()];
            let _ = match state_res::resolve(
                &RoomVersionId::Version6,
                &state_sets,
                state_sets
                    .iter()
                    .map(|map| {
                        store
                            .auth_event_ids(&room_id(), &map.values().cloned().collect::<Vec<_>>())
                            .unwrap()
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
struct TestStore<E: Event>(HashMap<EventId, Arc<E>>);

#[allow(unused)]
impl<E: Event> TestStore<E> {
    fn get_event(&self, room_id: &RoomId, event_id: &EventId) -> Result<Arc<E>> {
        self.0
            .get(event_id)
            .map(Arc::clone)
            .ok_or_else(|| Error::NotFound(format!("{} not found", event_id.to_string())))
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
    fn auth_event_ids(&self, room_id: &RoomId, event_ids: &[EventId]) -> Result<HashSet<EventId>> {
        let mut result = HashSet::new();
        let mut stack = event_ids.to_vec();

        // DFS for auth event chain
        while !stack.is_empty() {
            let ev_id = stack.pop().unwrap();
            if result.contains(&ev_id) {
                continue;
            }

            result.insert(ev_id.clone());

            let event = self.get_event(room_id, &ev_id)?;

            stack.extend(event.auth_events().cloned());
        }

        Ok(result)
    }

    /// Returns a Vec<EventId> representing the difference in auth chains of the given `events`.
    fn auth_chain_diff(
        &self,
        room_id: &RoomId,
        event_ids: Vec<Vec<EventId>>,
    ) -> Result<Vec<EventId>> {
        let mut auth_chain_sets = vec![];
        for ids in event_ids {
            // TODO state store `auth_event_ids` returns self in the event ids list
            // when an event returns `auth_event_ids` self is not contained
            let chain = self.auth_event_ids(room_id, &ids)?.into_iter().collect::<HashSet<_>>();
            auth_chain_sets.push(chain);
        }

        if let Some(first) = auth_chain_sets.first().cloned() {
            let common = auth_chain_sets
                .iter()
                .skip(1)
                .fold(first, |a, b| a.intersection(b).cloned().collect::<HashSet<EventId>>());

            Ok(auth_chain_sets.into_iter().flatten().filter(|id| !common.contains(id)).collect())
        } else {
            Ok(vec![])
        }
    }
}

impl TestStore<StateEvent> {
    fn set_up(&mut self) -> (StateMap<EventId>, StateMap<EventId>, StateMap<EventId>) {
        let create_event = to_pdu_event::<EventId>(
            "CREATE",
            alice(),
            EventType::RoomCreate,
            Some(""),
            json!({ "creator": alice() }),
            &[],
            &[],
        );
        let cre = create_event.event_id().clone();
        self.0.insert(cre.clone(), Arc::clone(&create_event));

        let alice_mem = to_pdu_event(
            "IMA",
            alice(),
            EventType::RoomMember,
            Some(alice().to_string().as_str()),
            member_content_join(),
            &[cre.clone()],
            &[cre.clone()],
        );
        self.0.insert(alice_mem.event_id().clone(), Arc::clone(&alice_mem));

        let join_rules = to_pdu_event(
            "IJR",
            alice(),
            EventType::RoomJoinRules,
            Some(""),
            json!({ "join_rule": JoinRule::Public }),
            &[cre.clone(), alice_mem.event_id().clone()],
            &[alice_mem.event_id().clone()],
        );
        self.0.insert(join_rules.event_id().clone(), join_rules.clone());

        // Bob and Charlie join at the same time, so there is a fork
        // this will be represented in the state_sets when we resolve
        let bob_mem = to_pdu_event(
            "IMB",
            bob(),
            EventType::RoomMember,
            Some(bob().to_string().as_str()),
            member_content_join(),
            &[cre.clone(), join_rules.event_id().clone()],
            &[join_rules.event_id().clone()],
        );
        self.0.insert(bob_mem.event_id().clone(), bob_mem.clone());

        let charlie_mem = to_pdu_event(
            "IMC",
            charlie(),
            EventType::RoomMember,
            Some(charlie().to_string().as_str()),
            member_content_join(),
            &[cre, join_rules.event_id().clone()],
            &[join_rules.event_id().clone()],
        );
        self.0.insert(charlie_mem.event_id().clone(), charlie_mem.clone());

        let state_at_bob = [&create_event, &alice_mem, &join_rules, &bob_mem]
            .iter()
            .map(|e| {
                (
                    (e.event_type().to_owned(), e.state_key().unwrap().to_owned()),
                    e.event_id().clone(),
                )
            })
            .collect::<StateMap<_>>();

        let state_at_charlie = [&create_event, &alice_mem, &join_rules, &charlie_mem]
            .iter()
            .map(|e| {
                (
                    (e.event_type().to_owned(), e.state_key().unwrap().to_owned()),
                    e.event_id().clone(),
                )
            })
            .collect::<StateMap<_>>();

        let expected = [&create_event, &alice_mem, &join_rules, &bob_mem, &charlie_mem]
            .iter()
            .map(|e| {
                (
                    (e.event_type().to_owned(), e.state_key().unwrap().to_owned()),
                    e.event_id().clone(),
                )
            })
            .collect::<StateMap<_>>();

        (state_at_bob, state_at_charlie, expected)
    }
}

fn event_id(id: &str) -> EventId {
    if id.contains('$') {
        return EventId::try_from(id).unwrap();
    }
    EventId::try_from(format!("${}:foo", id)).unwrap()
}

fn alice() -> UserId {
    UserId::try_from("@alice:foo").unwrap()
}
fn bob() -> UserId {
    UserId::try_from("@bob:foo").unwrap()
}
fn charlie() -> UserId {
    UserId::try_from("@charlie:foo").unwrap()
}
fn ella() -> UserId {
    UserId::try_from("@ella:foo").unwrap()
}

fn room_id() -> RoomId {
    RoomId::try_from("!test:foo").unwrap()
}

fn member_content_ban() -> JsonValue {
    serde_json::to_value(MemberEventContent::new(MembershipState::Ban)).unwrap()
}

fn member_content_join() -> JsonValue {
    serde_json::to_value(MemberEventContent::new(MembershipState::Join)).unwrap()
}

fn to_pdu_event<S>(
    id: &str,
    sender: UserId,
    ev_type: EventType,
    state_key: Option<&str>,
    content: JsonValue,
    auth_events: &[S],
    prev_events: &[S],
) -> Arc<StateEvent>
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
    Arc::new(StateEvent {
        event_id: EventId::try_from(id).unwrap(),
        rest: Pdu::RoomV3Pdu(RoomV3Pdu {
            room_id: room_id(),
            sender,
            origin_server_ts: MilliSecondsSinceUnixEpoch(ts.try_into().unwrap()),
            state_key,
            kind: ev_type,
            content,
            redacts: None,
            unsigned: btreemap! {},
            #[cfg(not(feature = "unstable-pre-spec"))]
            origin: "foo".into(),
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
fn INITIAL_EVENTS() -> HashMap<EventId, Arc<StateEvent>> {
    vec![
        to_pdu_event::<EventId>(
            "CREATE",
            alice(),
            EventType::RoomCreate,
            Some(""),
            json!({ "creator": alice() }),
            &[],
            &[],
        ),
        to_pdu_event(
            "IMA",
            alice(),
            EventType::RoomMember,
            Some(alice().to_string().as_str()),
            member_content_join(),
            &["CREATE"],
            &["CREATE"],
        ),
        to_pdu_event(
            "IPOWER",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({ "users": { alice().to_string(): 100 } }),
            &["CREATE", "IMA"],
            &["IMA"],
        ),
        to_pdu_event(
            "IJR",
            alice(),
            EventType::RoomJoinRules,
            Some(""),
            json!({ "join_rule": JoinRule::Public }),
            &["CREATE", "IMA", "IPOWER"],
            &["IPOWER"],
        ),
        to_pdu_event(
            "IMB",
            bob(),
            EventType::RoomMember,
            Some(bob().to_string().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "IPOWER"],
            &["IJR"],
        ),
        to_pdu_event(
            "IMC",
            charlie(),
            EventType::RoomMember,
            Some(charlie().to_string().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "IPOWER"],
            &["IMB"],
        ),
        to_pdu_event::<EventId>(
            "START",
            charlie(),
            EventType::RoomTopic,
            Some(""),
            json!({}),
            &[],
            &[],
        ),
        to_pdu_event::<EventId>(
            "END",
            charlie(),
            EventType::RoomTopic,
            Some(""),
            json!({}),
            &[],
            &[],
        ),
    ]
    .into_iter()
    .map(|ev| (ev.event_id().clone(), ev))
    .collect()
}

// all graphs start with these input events
#[allow(non_snake_case)]
fn BAN_STATE_SET() -> HashMap<EventId, Arc<StateEvent>> {
    vec![
        to_pdu_event(
            "PA",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({ "users": { alice(): 100, bob(): 50 } }),
            &["CREATE", "IMA", "IPOWER"], // auth_events
            &["START"],                   // prev_events
        ),
        to_pdu_event(
            "PB",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({ "users": { alice(): 100, bob(): 50 } }),
            &["CREATE", "IMA", "IPOWER"],
            &["END"],
        ),
        to_pdu_event(
            "MB",
            alice(),
            EventType::RoomMember,
            Some(ella().as_str()),
            member_content_ban(),
            &["CREATE", "IMA", "PB"],
            &["PA"],
        ),
        to_pdu_event(
            "IME",
            ella(),
            EventType::RoomMember,
            Some(ella().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "PA"],
            &["MB"],
        ),
    ]
    .into_iter()
    .map(|ev| (ev.event_id().clone(), ev))
    .collect()
}

mod event {
    use std::collections::BTreeMap;

    use js_int::UInt;
    use ruma_common::MilliSecondsSinceUnixEpoch;
    use ruma_events::{
        pdu::{EventHash, Pdu},
        EventType,
    };
    use ruma_identifiers::{EventId, RoomId, ServerName, ServerSigningKeyId, UserId};
    use ruma_state_res::Event;
    use serde::{Deserialize, Serialize};
    use serde_json::Value as JsonValue;

    impl Event for StateEvent {
        fn event_id(&self) -> &EventId {
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

        fn event_type(&self) -> &EventType {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => &ev.kind,
                Pdu::RoomV3Pdu(ev) => &ev.kind,
                #[cfg(not(feature = "unstable-exhaustive-types"))]
                _ => unreachable!("new PDU version"),
            }
        }

        fn content(&self) -> serde_json::Value {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => ev.content.clone(),
                Pdu::RoomV3Pdu(ev) => ev.content.clone(),
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

        fn prev_events(&self) -> Box<dyn DoubleEndedIterator<Item = &EventId> + '_> {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => Box::new(ev.prev_events.iter().map(|(id, _)| id)),
                Pdu::RoomV3Pdu(ev) => Box::new(ev.prev_events.iter()),
                #[cfg(not(feature = "unstable-exhaustive-types"))]
                _ => unreachable!("new PDU version"),
            }
        }

        fn depth(&self) -> &UInt {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => &ev.depth,
                Pdu::RoomV3Pdu(ev) => &ev.depth,
                #[cfg(not(feature = "unstable-exhaustive-types"))]
                _ => unreachable!("new PDU version"),
            }
        }

        fn auth_events(&self) -> Box<dyn DoubleEndedIterator<Item = &EventId> + '_> {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => Box::new(ev.auth_events.iter().map(|(id, _)| id)),
                Pdu::RoomV3Pdu(ev) => Box::new(ev.auth_events.iter()),
                #[cfg(not(feature = "unstable-exhaustive-types"))]
                _ => unreachable!("new PDU version"),
            }
        }

        fn redacts(&self) -> Option<&EventId> {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => ev.redacts.as_ref(),
                Pdu::RoomV3Pdu(ev) => ev.redacts.as_ref(),
                #[cfg(not(feature = "unstable-exhaustive-types"))]
                _ => unreachable!("new PDU version"),
            }
        }

        fn hashes(&self) -> &EventHash {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => &ev.hashes,
                Pdu::RoomV3Pdu(ev) => &ev.hashes,
                #[cfg(not(feature = "unstable-exhaustive-types"))]
                _ => unreachable!("new PDU version"),
            }
        }

        fn signatures(&self) -> BTreeMap<Box<ServerName>, BTreeMap<ServerSigningKeyId, String>> {
            match &self.rest {
                Pdu::RoomV1Pdu(_) => maplit::btreemap! {},
                Pdu::RoomV3Pdu(ev) => ev.signatures.clone(),
                #[cfg(not(feature = "unstable-exhaustive-types"))]
                _ => unreachable!("new PDU version"),
            }
        }

        fn unsigned(&self) -> &BTreeMap<String, JsonValue> {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => &ev.unsigned,
                Pdu::RoomV3Pdu(ev) => &ev.unsigned,
                #[cfg(not(feature = "unstable-exhaustive-types"))]
                _ => unreachable!("new PDU version"),
            }
        }
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct StateEvent {
        pub event_id: EventId,
        #[serde(flatten)]
        pub rest: Pdu,
    }
}
