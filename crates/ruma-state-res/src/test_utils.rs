use std::{
    borrow::Borrow,
    collections::{BTreeMap, HashMap, HashSet},
    convert::TryInto,
    sync::{
        atomic::{AtomicU64, Ordering::SeqCst},
        Arc,
    },
};

use js_int::{int, uint};
use ruma_common::{
    event_id,
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
use serde_json::{
    json,
    value::{to_raw_value as to_raw_json_value, RawValue as RawJsonValue},
};
use tracing::info;

use crate::{auth_types_for_event, Error, Event, Result, StateMap};

pub use event::OriginalStateEvent;

static SERVER_TIMESTAMP: AtomicU64 = AtomicU64::new(0);

pub fn do_check(
    events: &[Arc<OriginalStateEvent>],
    edges: Vec<Vec<Box<EventId>>>,
    expected_state_ids: Vec<Box<EventId>>,
) {
    // To activate logging use `RUST_LOG=debug cargo t`

    let init_events = INITIAL_EVENTS();

    let mut store = TestStore(
        init_events
            .values()
            .chain(events)
            .map(|ev| (ev.event_id().to_owned(), ev.clone()))
            .collect(),
    );

    // This will be lexi_topo_sorted for resolution
    let mut graph = HashMap::new();
    // This is the same as in `resolve` event_id -> OriginalStateEvent
    let mut fake_event_map = HashMap::new();

    // Create the DB of events that led up to this point
    // TODO maybe clean up some of these clones it is just tests but...
    for ev in init_events.values().chain(events) {
        graph.insert(ev.event_id().to_owned(), HashSet::new());
        fake_event_map.insert(ev.event_id().to_owned(), ev.clone());
    }

    for pair in INITIAL_EDGES().windows(2) {
        if let [a, b] = &pair {
            graph.entry(a.to_owned()).or_insert_with(HashSet::new).insert(b.clone());
        }
    }

    for edge_list in edges {
        for pair in edge_list.windows(2) {
            if let [a, b] = &pair {
                graph.entry(a.to_owned()).or_insert_with(HashSet::new).insert(b.clone());
            }
        }
    }

    // event_id -> OriginalStateEvent
    let mut event_map: HashMap<Box<EventId>, Arc<OriginalStateEvent>> = HashMap::new();
    // event_id -> StateMap<Box<EventId>>
    let mut state_at_event: HashMap<Box<EventId>, StateMap<Box<EventId>>> = HashMap::new();

    // Resolve the current state and add it to the state_at_event map then continue
    // on in "time"
    for node in crate::lexicographical_topological_sort(&graph, |_id| {
        Ok((int!(0), MilliSecondsSinceUnixEpoch(uint!(0))))
    })
    .unwrap()
    {
        let fake_event = fake_event_map.get(&node).unwrap();
        let event_id = fake_event.event_id().to_owned();

        let prev_events = graph.get(&node).unwrap();

        let state_before: StateMap<Box<EventId>> = if prev_events.is_empty() {
            HashMap::new()
        } else if prev_events.len() == 1 {
            state_at_event.get(prev_events.iter().next().unwrap()).unwrap().clone()
        } else {
            let state_sets =
                prev_events.iter().filter_map(|k| state_at_event.get(k)).collect::<Vec<_>>();

            info!(
                "{:#?}",
                state_sets
                    .iter()
                    .map(|map| map
                        .iter()
                        .map(|((ty, key), id)| format!("(({}{:?}), {})", ty, key, id))
                        .collect::<Vec<_>>())
                    .collect::<Vec<_>>()
            );

            let auth_chain_sets = state_sets
                .iter()
                .map(|map| {
                    store.auth_event_ids(room_id(), map.values().cloned().collect()).unwrap()
                })
                .collect();

            let resolved = crate::resolve(&RoomVersionId::V6, state_sets, auth_chain_sets, |id| {
                event_map.get(id).map(Arc::clone)
            });
            match resolved {
                Ok(state) => state,
                Err(e) => panic!("resolution for {} failed: {}", node, e),
            }
        };

        let mut state_after = state_before.clone();

        let ty = fake_event.event_type().to_owned();
        let key = fake_event.state_key().unwrap().to_owned();
        state_after.insert((ty, key), event_id.to_owned());

        let auth_types = auth_types_for_event(
            fake_event.event_type(),
            fake_event.sender(),
            fake_event.state_key(),
            fake_event.content(),
        )
        .unwrap();

        let mut auth_events = vec![];
        for key in auth_types {
            if state_before.contains_key(&key) {
                auth_events.push(state_before[&key].clone());
            }
        }

        // TODO The event is just remade, adding the auth_events and prev_events here
        // the `to_pdu_event` was split into `init` and the fn below, could be better
        let e = fake_event;
        let ev_id = e.event_id();
        let event = to_pdu_event(
            e.event_id().as_str(),
            e.sender().to_owned(),
            e.event_type().clone(),
            e.state_key(),
            e.content().to_owned(),
            &auth_events,
            &prev_events.iter().cloned().collect::<Vec<_>>(),
        );

        // We have to update our store, an actual user of this lib would
        // be giving us state from a DB.
        store.0.insert(ev_id.to_owned(), event.clone());

        state_at_event.insert(node, state_after);
        event_map.insert(event_id.to_owned(), Arc::clone(store.0.get(ev_id).unwrap()));
    }

    let mut expected_state = StateMap::new();
    for node in expected_state_ids {
        let ev = event_map.get(&node).unwrap_or_else(|| {
            panic!(
                "{} not found in {:?}",
                node,
                event_map.keys().map(ToString::to_string).collect::<Vec<_>>()
            )
        });

        let key = (ev.event_type().to_owned(), ev.state_key().unwrap().to_owned());

        expected_state.insert(key, node);
    }

    let start_state = state_at_event.get(event_id!("$START:foo")).unwrap();

    let end_state = state_at_event
        .get(event_id!("$END:foo"))
        .unwrap()
        .iter()
        .filter(|(k, v)| {
            expected_state.contains_key(k)
                || start_state.get(k) != Some(*v)
                // Filter out the dummy messages events.
                // These act as points in time where there should be a known state to
                // test against.
                && **k != (RoomEventType::RoomMessage, "dummy".to_owned())
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<StateMap<Box<EventId>>>();

    assert_eq!(expected_state, end_state);
}

#[allow(clippy::exhaustive_structs)]
pub struct TestStore<E: Event>(pub HashMap<Box<EventId>, Arc<E>>);

impl<E: Event> TestStore<E> {
    pub fn get_event(&self, _: &RoomId, event_id: &EventId) -> Result<Arc<E>> {
        self.0
            .get(event_id)
            .map(Arc::clone)
            .ok_or_else(|| Error::NotFound(format!("{} not found", event_id)))
    }

    /// Returns a Vec of the related auth events to the given `event`.
    pub fn auth_event_ids(
        &self,
        room_id: &RoomId,
        event_ids: Vec<E::Id>,
    ) -> Result<HashSet<E::Id>> {
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
}

// A StateStore implementation for testing
#[allow(clippy::type_complexity)]
impl TestStore<OriginalStateEvent> {
    pub fn set_up(
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

pub fn event_id(id: &str) -> Box<EventId> {
    if id.contains('$') {
        return id.try_into().unwrap();
    }

    format!("${}:foo", id).try_into().unwrap()
}

pub fn alice() -> Box<UserId> {
    user_id!("@alice:foo").to_owned()
}

pub fn bob() -> Box<UserId> {
    user_id!("@bob:foo").to_owned()
}

pub fn charlie() -> Box<UserId> {
    user_id!("@charlie:foo").to_owned()
}

pub fn ella() -> Box<UserId> {
    user_id!("@ella:foo").to_owned()
}

pub fn zara() -> Box<UserId> {
    user_id!("@zara:foo").to_owned()
}

pub fn room_id() -> &'static RoomId {
    room_id!("!test:foo")
}

pub fn member_content_ban() -> Box<RawJsonValue> {
    to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Ban)).unwrap()
}

pub fn member_content_join() -> Box<RawJsonValue> {
    to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Join)).unwrap()
}

pub fn to_init_pdu_event(
    id: &str,
    sender: Box<UserId>,
    ev_type: RoomEventType,
    state_key: Option<&str>,
    content: Box<RawJsonValue>,
) -> Arc<OriginalStateEvent> {
    let ts = SERVER_TIMESTAMP.fetch_add(1, SeqCst);
    let id = if id.contains('$') { id.to_owned() } else { format!("${}:foo", id) };

    let state_key = state_key.map(ToOwned::to_owned);
    Arc::new(OriginalStateEvent {
        event_id: id.try_into().unwrap(),
        rest: Pdu::RoomV3Pdu(RoomV3Pdu {
            room_id: room_id().to_owned(),
            sender,
            origin_server_ts: MilliSecondsSinceUnixEpoch(ts.try_into().unwrap()),
            state_key,
            kind: ev_type,
            content,
            redacts: None,
            unsigned: BTreeMap::new(),
            auth_events: vec![],
            prev_events: vec![],
            depth: uint!(0),
            hashes: EventHash::new("".to_owned()),
            signatures: BTreeMap::new(),
        }),
    })
}

pub fn to_pdu_event<S>(
    id: &str,
    sender: Box<UserId>,
    ev_type: RoomEventType,
    state_key: Option<&str>,
    content: Box<RawJsonValue>,
    auth_events: &[S],
    prev_events: &[S],
) -> Arc<OriginalStateEvent>
where
    S: AsRef<str>,
{
    let ts = SERVER_TIMESTAMP.fetch_add(1, SeqCst);
    let id = if id.contains('$') { id.to_owned() } else { format!("${}:foo", id) };
    let auth_events = auth_events.iter().map(AsRef::as_ref).map(event_id).collect::<Vec<_>>();
    let prev_events = prev_events.iter().map(AsRef::as_ref).map(event_id).collect::<Vec<_>>();

    let state_key = state_key.map(ToOwned::to_owned);
    Arc::new(OriginalStateEvent {
        event_id: id.try_into().unwrap(),
        rest: Pdu::RoomV3Pdu(RoomV3Pdu {
            room_id: room_id().to_owned(),
            sender,
            origin_server_ts: MilliSecondsSinceUnixEpoch(ts.try_into().unwrap()),
            state_key,
            kind: ev_type,
            content,
            redacts: None,
            unsigned: BTreeMap::new(),
            auth_events,
            prev_events,
            depth: uint!(0),
            hashes: EventHash::new("".to_owned()),
            signatures: BTreeMap::new(),
        }),
    })
}

// all graphs start with these input events
#[allow(non_snake_case)]
pub fn INITIAL_EVENTS() -> HashMap<Box<EventId>, Arc<OriginalStateEvent>> {
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
            RoomEventType::RoomMessage,
            Some("dummy"),
            to_raw_json_value(&json!({})).unwrap(),
            &[],
            &[],
        ),
        to_pdu_event::<&EventId>(
            "END",
            charlie(),
            RoomEventType::RoomMessage,
            Some("dummy"),
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
pub fn INITIAL_EVENTS_CREATE_ROOM() -> HashMap<Box<EventId>, Arc<OriginalStateEvent>> {
    vec![to_pdu_event::<&EventId>(
        "CREATE",
        alice(),
        RoomEventType::RoomCreate,
        Some(""),
        to_raw_json_value(&json!({ "creator": alice() })).unwrap(),
        &[],
        &[],
    )]
    .into_iter()
    .map(|ev| (ev.event_id().to_owned(), ev))
    .collect()
}

#[allow(non_snake_case)]
pub fn INITIAL_EDGES() -> Vec<Box<EventId>> {
    vec!["START", "IMC", "IMB", "IJR", "IPOWER", "IMA", "CREATE"]
        .into_iter()
        .map(event_id)
        .collect::<Vec<_>>()
}

pub mod event {
    use ruma_common::{
        events::{pdu::Pdu, RoomEventType},
        EventId, MilliSecondsSinceUnixEpoch, RoomId, UserId,
    };
    use serde::{Deserialize, Serialize};
    use serde_json::value::RawValue as RawJsonValue;

    use crate::Event;

    impl Event for OriginalStateEvent {
        type Id = Box<EventId>;

        fn event_id(&self) -> &Self::Id {
            &self.event_id
        }

        fn room_id(&self) -> &RoomId {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => &ev.room_id,
                Pdu::RoomV3Pdu(ev) => &ev.room_id,
                #[allow(unreachable_patterns)]
                _ => unreachable!("new PDU version"),
            }
        }

        fn sender(&self) -> &UserId {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => &ev.sender,
                Pdu::RoomV3Pdu(ev) => &ev.sender,
                #[allow(unreachable_patterns)]
                _ => unreachable!("new PDU version"),
            }
        }

        fn event_type(&self) -> &RoomEventType {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => &ev.kind,
                Pdu::RoomV3Pdu(ev) => &ev.kind,
                #[allow(unreachable_patterns)]
                _ => unreachable!("new PDU version"),
            }
        }

        fn content(&self) -> &RawJsonValue {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => &ev.content,
                Pdu::RoomV3Pdu(ev) => &ev.content,
                #[allow(unreachable_patterns)]
                _ => unreachable!("new PDU version"),
            }
        }

        fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => ev.origin_server_ts,
                Pdu::RoomV3Pdu(ev) => ev.origin_server_ts,
                #[allow(unreachable_patterns)]
                _ => unreachable!("new PDU version"),
            }
        }

        fn state_key(&self) -> Option<&str> {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => ev.state_key.as_deref(),
                Pdu::RoomV3Pdu(ev) => ev.state_key.as_deref(),
                #[allow(unreachable_patterns)]
                _ => unreachable!("new PDU version"),
            }
        }

        fn prev_events(&self) -> Box<dyn DoubleEndedIterator<Item = &Self::Id> + '_> {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => Box::new(ev.prev_events.iter().map(|(id, _)| id)),
                Pdu::RoomV3Pdu(ev) => Box::new(ev.prev_events.iter()),
                #[allow(unreachable_patterns)]
                _ => unreachable!("new PDU version"),
            }
        }

        fn auth_events(&self) -> Box<dyn DoubleEndedIterator<Item = &Self::Id> + '_> {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => Box::new(ev.auth_events.iter().map(|(id, _)| id)),
                Pdu::RoomV3Pdu(ev) => Box::new(ev.auth_events.iter()),
                #[allow(unreachable_patterns)]
                _ => unreachable!("new PDU version"),
            }
        }

        fn redacts(&self) -> Option<&Self::Id> {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => ev.redacts.as_ref(),
                Pdu::RoomV3Pdu(ev) => ev.redacts.as_ref(),
                #[allow(unreachable_patterns)]
                _ => unreachable!("new PDU version"),
            }
        }
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[allow(clippy::exhaustive_structs)]
    pub struct OriginalStateEvent {
        pub event_id: Box<EventId>,
        #[serde(flatten)]
        pub rest: Pdu,
    }

    //impl OriginalStateEvent {
    //    pub fn state_key(&self) -> &str {
    //        match &self.rest {
    //            Pdu::RoomV1Pdu(ev) => ev.state_key.as_ref().unwrap(),
    //            Pdu::RoomV3Pdu(ev) => ev.state_key.as_ref().unwrap(),
    //            #[cfg(not(feature = "unstable-exhaustive-types"))]
    //            _ => unreachable!("new PDU version"),
    //        }
    //    }
    //}
}
