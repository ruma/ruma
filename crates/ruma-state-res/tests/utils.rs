#![allow(clippy::or_fun_call, clippy::expect_fun_call, dead_code)]

use std::{
    collections::{BTreeMap, BTreeSet},
    convert::{TryFrom, TryInto},
    sync::{
        atomic::{AtomicU64, Ordering::SeqCst},
        Arc, Once,
    },
};

use js_int::uint;
use maplit::{btreemap, btreeset};
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
use ruma_state_res::{auth_types_for_event, Error, Event, Result, StateMap, StateResolution};
use serde_json::{json, Value as JsonValue};
use tracing::info;
use tracing_subscriber as tracer;

pub use event::StateEvent;

pub static LOGGER: Once = Once::new();

static SERVER_TIMESTAMP: AtomicU64 = AtomicU64::new(0);

pub fn do_check(
    events: &[Arc<StateEvent>],
    edges: Vec<Vec<EventId>>,
    expected_state_ids: Vec<EventId>,
) {
    // To activate logging use `RUST_LOG=debug cargo t`
    // The logger is initialized in the `INITIAL_EVENTS` function.

    let init_events = INITIAL_EVENTS();

    let mut store = TestStore(
        init_events.values().chain(events).map(|ev| (ev.event_id().clone(), ev.clone())).collect(),
    );

    // This will be lexi_topo_sorted for resolution
    let mut graph = BTreeMap::new();
    // This is the same as in `resolve` event_id -> StateEvent
    let mut fake_event_map = BTreeMap::new();

    // Create the DB of events that led up to this point
    // TODO maybe clean up some of these clones it is just tests but...
    for ev in init_events.values().chain(events) {
        graph.insert(ev.event_id().clone(), btreeset![]);
        fake_event_map.insert(ev.event_id().clone(), ev.clone());
    }

    for pair in INITIAL_EDGES().windows(2) {
        if let [a, b] = &pair {
            graph.entry(a.clone()).or_insert(btreeset![]).insert(b.clone());
        }
    }

    for edge_list in edges {
        for pair in edge_list.windows(2) {
            if let [a, b] = &pair {
                graph.entry(a.clone()).or_insert(btreeset![]).insert(b.clone());
            }
        }
    }

    // event_id -> StateEvent
    let mut event_map: BTreeMap<EventId, Arc<StateEvent>> = BTreeMap::new();
    // event_id -> StateMap<EventId>
    let mut state_at_event: BTreeMap<EventId, StateMap<EventId>> = BTreeMap::new();

    // Resolve the current state and add it to the state_at_event map then continue
    // on in "time"
    for node in StateResolution::lexicographical_topological_sort(&graph, |id| {
        (0, MilliSecondsSinceUnixEpoch(uint!(0)), id.clone())
    }) {
        let fake_event = fake_event_map.get(&node).unwrap();
        let event_id = fake_event.event_id().clone();

        let prev_events = graph.get(&node).unwrap();

        let state_before: StateMap<EventId> = if prev_events.is_empty() {
            BTreeMap::new()
        } else if prev_events.len() == 1 {
            state_at_event.get(prev_events.iter().next().unwrap()).unwrap().clone()
        } else {
            let state_sets = prev_events
                .iter()
                .filter_map(|k| state_at_event.get(k))
                .cloned()
                .collect::<Vec<_>>();

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

            let resolved = StateResolution::resolve(
                &room_id(),
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
                &mut event_map,
            );
            match resolved {
                Ok(state) => state,
                Err(e) => panic!("resolution for {} failed: {}", node, e),
            }
        };

        let mut state_after = state_before.clone();

        let ty = fake_event.kind();
        let key = fake_event.state_key();
        state_after.insert((ty, key), event_id.clone());

        let auth_types = auth_types_for_event(
            &fake_event.kind(),
            fake_event.sender(),
            Some(fake_event.state_key()),
            fake_event.content(),
        );

        let mut auth_events = vec![];
        for key in auth_types {
            if state_before.contains_key(&key) {
                auth_events.push(state_before[&key].clone())
            }
        }

        // TODO The event is just remade, adding the auth_events and prev_events here
        // the `to_pdu_event` was split into `init` and the fn below, could be better
        let e = fake_event;
        let ev_id = e.event_id().clone();
        let event = to_pdu_event(
            e.event_id().as_str(),
            e.sender().clone(),
            e.kind().clone(),
            Some(&e.state_key()),
            e.content(),
            &auth_events,
            &prev_events.iter().cloned().collect::<Vec<_>>(),
        );

        // We have to update our store, an actual user of this lib would
        // be giving us state from a DB.
        store.0.insert(ev_id.clone(), event.clone());

        state_at_event.insert(node, state_after);
        event_map.insert(event_id.clone(), Arc::clone(store.0.get(&ev_id).unwrap()));
    }

    let mut expected_state = StateMap::new();
    for node in expected_state_ids {
        let ev = event_map.get(&node).expect(&format!(
            "{} not found in {:?}",
            node.to_string(),
            event_map.keys().map(ToString::to_string).collect::<Vec<_>>(),
        ));

        let key = (ev.kind(), ev.state_key());

        expected_state.insert(key, node);
    }

    let start_state = state_at_event.get(&event_id("$START:foo")).unwrap();

    let end_state = state_at_event
        .get(&event_id("$END:foo"))
        .unwrap()
        .iter()
        .filter(|(k, v)| {
            expected_state.contains_key(k)
                || start_state.get(k) != Some(*v)
                // Filter out the dummy messages events.
                // These act as points in time where there should be a known state to
                // test against.
                && **k != (EventType::RoomMessage, "dummy".to_string())
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<StateMap<EventId>>();

    assert_eq!(expected_state, end_state);
}

pub struct TestStore<E: Event>(pub BTreeMap<EventId, Arc<E>>);

#[allow(unused)]
impl<E: Event> TestStore<E> {
    pub fn get_event(&self, room_id: &RoomId, event_id: &EventId) -> Result<Arc<E>> {
        self.0
            .get(event_id)
            .map(Arc::clone)
            .ok_or_else(|| Error::NotFound(format!("{} not found", event_id.to_string())))
    }

    /// Returns the events that correspond to the `event_ids` sorted in the same order.
    pub fn get_events(&self, room_id: &RoomId, event_ids: &[EventId]) -> Result<Vec<Arc<E>>> {
        let mut events = vec![];
        for id in event_ids {
            events.push(self.get_event(room_id, id)?);
        }
        Ok(events)
    }

    /// Returns a Vec of the related auth events to the given `event`.
    pub fn auth_event_ids(&self, room_id: &RoomId, event_ids: &[EventId]) -> Result<Vec<EventId>> {
        let mut result = vec![];
        let mut stack = event_ids.to_vec();

        // DFS for auth event chain
        while !stack.is_empty() {
            let ev_id = stack.pop().unwrap();
            if result.contains(&ev_id) {
                continue;
            }

            result.push(ev_id.clone());

            let event = self.get_event(room_id, &ev_id)?;

            stack.extend(event.auth_events().clone());
        }

        Ok(result)
    }

    /// Returns a Vec<EventId> representing the difference in auth chains of the given `events`.
    pub fn auth_chain_diff(
        &self,
        room_id: &RoomId,
        event_ids: Vec<Vec<EventId>>,
    ) -> Result<Vec<EventId>> {
        use itertools::Itertools;
        let mut chains = vec![];
        for ids in event_ids {
            // TODO state store `auth_event_ids` returns self in the event ids list
            // when an event returns `auth_event_ids` self is not contained
            let chain = self.auth_event_ids(room_id, &ids)?.into_iter().collect::<BTreeSet<_>>();
            chains.push(chain);
        }

        if let Some(chain) = chains.first().cloned() {
            let rest = chains.iter().skip(1).flatten().cloned().collect();
            let common = chain.intersection(&rest).collect::<Vec<_>>();

            Ok(chains.into_iter().flatten().filter(|id| !common.contains(&id)).dedup().collect())
        } else {
            Ok(vec![])
        }
    }
}

pub fn event_id(id: &str) -> EventId {
    if id.contains('$') {
        return EventId::try_from(id).unwrap();
    }
    EventId::try_from(format!("${}:foo", id)).unwrap()
}

pub fn alice() -> UserId {
    UserId::try_from("@alice:foo").unwrap()
}
pub fn bob() -> UserId {
    UserId::try_from("@bob:foo").unwrap()
}
pub fn charlie() -> UserId {
    UserId::try_from("@charlie:foo").unwrap()
}
pub fn ella() -> UserId {
    UserId::try_from("@ella:foo").unwrap()
}
pub fn zara() -> UserId {
    UserId::try_from("@zara:foo").unwrap()
}

pub fn room_id() -> RoomId {
    RoomId::try_from("!test:foo").unwrap()
}

pub fn member_content_ban() -> JsonValue {
    serde_json::to_value(MemberEventContent::new(MembershipState::Ban)).unwrap()
}

pub fn member_content_join() -> JsonValue {
    serde_json::to_value(MemberEventContent::new(MembershipState::Join)).unwrap()
}

pub fn to_init_pdu_event(
    id: &str,
    sender: UserId,
    ev_type: EventType,
    state_key: Option<&str>,
    content: JsonValue,
) -> Arc<StateEvent> {
    let ts = SERVER_TIMESTAMP.fetch_add(1, SeqCst);
    let id = if id.contains('$') { id.to_string() } else { format!("${}:foo", id) };

    let state_key = state_key.map(ToString::to_string);
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
            auth_events: vec![],
            prev_events: vec![],
            depth: uint!(0),
            hashes: EventHash { sha256: "".into() },
            signatures: btreemap! {},
        }),
    })
}

pub fn to_pdu_event<S>(
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
    let ts = SERVER_TIMESTAMP.fetch_add(1, SeqCst);
    let id = if id.contains('$') { id.to_string() } else { format!("${}:foo", id) };
    let auth_events = auth_events.iter().map(AsRef::as_ref).map(event_id).collect::<Vec<_>>();
    let prev_events = prev_events.iter().map(AsRef::as_ref).map(event_id).collect::<Vec<_>>();

    let state_key = state_key.map(ToString::to_string);
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
            hashes: EventHash { sha256: "".into() },
            signatures: btreemap! {},
        }),
    })
}

// all graphs start with these input events
#[allow(non_snake_case)]
pub fn INITIAL_EVENTS() -> BTreeMap<EventId, Arc<StateEvent>> {
    // this is always called so we can init the logger here
    let _ = LOGGER
        .call_once(|| tracer::fmt().with_env_filter(tracer::EnvFilter::from_default_env()).init());

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
            EventType::RoomMessage,
            Some("dummy"),
            json!({}),
            &[],
            &[],
        ),
        to_pdu_event::<EventId>(
            "END",
            charlie(),
            EventType::RoomMessage,
            Some("dummy"),
            json!({}),
            &[],
            &[],
        ),
    ]
    .into_iter()
    .map(|ev| (ev.event_id().clone(), ev))
    .collect()
}

#[allow(non_snake_case)]
pub fn INITIAL_EDGES() -> Vec<EventId> {
    vec!["START", "IMC", "IMB", "IJR", "IPOWER", "IMA", "CREATE"]
        .into_iter()
        .map(event_id)
        .collect::<Vec<_>>()
}

pub mod event {
    use std::collections::BTreeMap;

    use js_int::UInt;
    use ruma_events::{
        exports::ruma_common::MilliSecondsSinceUnixEpoch,
        pdu::{EventHash, Pdu},
        room::member::{MemberEventContent, MembershipState},
        EventType,
    };
    use ruma_identifiers::{
        EventId, RoomId, RoomVersionId, ServerName, ServerSigningKeyId, UserId,
    };
    use ruma_serde::CanonicalJsonObject;
    use ruma_state_res::Event;
    use serde::{Deserialize, Serialize};
    use serde_json::Value as JsonValue;

    impl Event for StateEvent {
        fn event_id(&self) -> &EventId {
            self.event_id()
        }

        fn room_id(&self) -> &RoomId {
            self.room_id()
        }

        fn sender(&self) -> &UserId {
            self.sender()
        }
        fn kind(&self) -> EventType {
            self.kind()
        }

        fn content(&self) -> serde_json::Value {
            self.content()
        }
        fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch {
            *self.origin_server_ts()
        }

        fn state_key(&self) -> Option<String> {
            Some(self.state_key())
        }
        fn prev_events(&self) -> Vec<EventId> {
            self.prev_event_ids()
        }
        fn depth(&self) -> &UInt {
            self.depth()
        }
        fn auth_events(&self) -> Vec<EventId> {
            self.auth_events()
        }
        fn redacts(&self) -> Option<&EventId> {
            self.redacts()
        }
        fn hashes(&self) -> &EventHash {
            self.hashes()
        }
        fn signatures(&self) -> BTreeMap<Box<ServerName>, BTreeMap<ServerSigningKeyId, String>> {
            self.signatures()
        }
        fn unsigned(&self) -> &BTreeMap<String, JsonValue> {
            self.unsigned()
        }
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct StateEvent {
        pub event_id: EventId,
        #[serde(flatten)]
        pub rest: Pdu,
    }

    impl StateEvent {
        pub fn from_id_value(id: EventId, json: serde_json::Value) -> serde_json::Result<Self> {
            Ok(Self { event_id: id, rest: Pdu::RoomV3Pdu(serde_json::from_value(json)?) })
        }

        pub fn from_id_canon_obj(
            id: EventId,
            json: CanonicalJsonObject,
        ) -> serde_json::Result<Self> {
            Ok(Self {
                event_id: id,
                // TODO: this is unfortunate (from_value(to_value(json)))...
                rest: Pdu::RoomV3Pdu(serde_json::from_value(serde_json::to_value(json)?)?),
            })
        }

        pub fn is_power_event(&self) -> bool {
            match &self.rest {
                Pdu::RoomV1Pdu(event) => match event.kind {
                    EventType::RoomPowerLevels
                    | EventType::RoomJoinRules
                    | EventType::RoomCreate => event.state_key == Some("".into()),
                    EventType::RoomMember => {
                        // TODO fix clone
                        if let Ok(content) =
                            serde_json::from_value::<MemberEventContent>(event.content.clone())
                        {
                            if [MembershipState::Leave, MembershipState::Ban]
                                .contains(&content.membership)
                            {
                                return event.sender.as_str()
                                        // TODO is None here a failure
                                        != event.state_key.as_deref().unwrap_or("NOT A STATE KEY");
                            }
                        }

                        false
                    }
                    _ => false,
                },
                Pdu::RoomV3Pdu(event) => event.state_key == Some("".into()),
            }
        }
        pub fn deserialize_content<C: serde::de::DeserializeOwned>(&self) -> serde_json::Result<C> {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => serde_json::from_value(ev.content.clone()),
                Pdu::RoomV3Pdu(ev) => serde_json::from_value(ev.content.clone()),
            }
        }
        pub fn origin_server_ts(&self) -> &MilliSecondsSinceUnixEpoch {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => &ev.origin_server_ts,
                Pdu::RoomV3Pdu(ev) => &ev.origin_server_ts,
            }
        }
        pub fn event_id(&self) -> &EventId {
            &self.event_id
        }

        pub fn sender(&self) -> &UserId {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => &ev.sender,
                Pdu::RoomV3Pdu(ev) => &ev.sender,
            }
        }

        pub fn redacts(&self) -> Option<&EventId> {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => ev.redacts.as_ref(),
                Pdu::RoomV3Pdu(ev) => ev.redacts.as_ref(),
            }
        }

        pub fn room_id(&self) -> &RoomId {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => &ev.room_id,
                Pdu::RoomV3Pdu(ev) => &ev.room_id,
            }
        }
        pub fn kind(&self) -> EventType {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => ev.kind.clone(),
                Pdu::RoomV3Pdu(ev) => ev.kind.clone(),
            }
        }
        pub fn state_key(&self) -> String {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => ev.state_key.clone().unwrap(),
                Pdu::RoomV3Pdu(ev) => ev.state_key.clone().unwrap(),
            }
        }

        #[cfg(not(feature = "unstable-pre-spec"))]
        pub fn origin(&self) -> String {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => ev.origin.clone(),
                Pdu::RoomV3Pdu(ev) => ev.origin.clone(),
            }
        }

        pub fn prev_event_ids(&self) -> Vec<EventId> {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => ev.prev_events.iter().map(|(id, _)| id).cloned().collect(),
                Pdu::RoomV3Pdu(ev) => ev.prev_events.clone(),
            }
        }

        pub fn auth_events(&self) -> Vec<EventId> {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => ev.auth_events.iter().map(|(id, _)| id).cloned().collect(),
                Pdu::RoomV3Pdu(ev) => ev.auth_events.to_vec(),
            }
        }

        pub fn content(&self) -> serde_json::Value {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => ev.content.clone(),
                Pdu::RoomV3Pdu(ev) => ev.content.clone(),
            }
        }

        pub fn unsigned(&self) -> &BTreeMap<String, serde_json::Value> {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => &ev.unsigned,
                Pdu::RoomV3Pdu(ev) => &ev.unsigned,
            }
        }

        pub fn signatures(
            &self,
        ) -> BTreeMap<Box<ServerName>, BTreeMap<ServerSigningKeyId, String>> {
            match &self.rest {
                Pdu::RoomV1Pdu(_) => maplit::btreemap! {},
                Pdu::RoomV3Pdu(ev) => ev.signatures.clone(),
            }
        }

        pub fn hashes(&self) -> &EventHash {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => &ev.hashes,
                Pdu::RoomV3Pdu(ev) => &ev.hashes,
            }
        }

        pub fn depth(&self) -> &UInt {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => &ev.depth,
                Pdu::RoomV3Pdu(ev) => &ev.depth,
            }
        }

        pub fn is_type_and_key(&self, ev_type: EventType, state_key: &str) -> bool {
            match &self.rest {
                Pdu::RoomV1Pdu(ev) => {
                    ev.kind == ev_type && ev.state_key.as_deref() == Some(state_key)
                }
                Pdu::RoomV3Pdu(ev) => {
                    ev.kind == ev_type && ev.state_key.as_deref() == Some(state_key)
                }
            }
        }

        /// Returns the room version this event is formatted for.
        ///
        /// Currently either version 1 or 6 is returned, 6 represents
        /// version 3 and above.
        pub fn room_version(&self) -> RoomVersionId {
            // TODO: We have to know the actual room version this is not sufficient
            match self.rest {
                Pdu::RoomV1Pdu(_) => RoomVersionId::Version1,
                Pdu::RoomV3Pdu(_) => RoomVersionId::Version6,
            }
        }
    }
}
