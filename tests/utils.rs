#![allow(clippy::or_fun_call, clippy::expect_fun_call, dead_code)]

use std::{
    collections::BTreeMap,
    convert::TryFrom,
    sync::{Arc, Once},
    time::UNIX_EPOCH,
};

use ruma::{
    events::{
        room::{
            join_rules::JoinRule,
            member::{MemberEventContent, MembershipState},
        },
        EventType,
    },
    identifiers::{EventId, RoomId, RoomVersionId, UserId},
};
use serde_json::{json, Value as JsonValue};
use state_res::{Error, Event, Result, StateMap, StateResolution, StateStore};
use tracing_subscriber as tracer;

pub use event::StateEvent;

pub static LOGGER: Once = Once::new();

static mut SERVER_TIMESTAMP: i32 = 0;

pub fn do_check(
    events: &[Arc<StateEvent>],
    edges: Vec<Vec<EventId>>,
    expected_state_ids: Vec<EventId>,
) {
    // to activate logging use `RUST_LOG=debug cargo t`
    let _ = LOGGER.call_once(|| {
        tracer::fmt()
            .with_env_filter(tracer::EnvFilter::from_default_env())
            .init()
    });

    let mut store = TestStore(
        INITIAL_EVENTS()
            .values()
            .chain(events)
            .map(|ev| (ev.event_id().clone(), ev.clone()))
            .collect(),
    );

    // This will be lexi_topo_sorted for resolution
    let mut graph = BTreeMap::new();
    // this is the same as in `resolve` event_id -> StateEvent
    let mut fake_event_map = BTreeMap::new();

    // create the DB of events that led up to this point
    // TODO maybe clean up some of these clones it is just tests but...
    for ev in INITIAL_EVENTS().values().chain(events) {
        graph.insert(ev.event_id().clone(), vec![]);
        fake_event_map.insert(ev.event_id().clone(), ev.clone());
    }

    for pair in INITIAL_EDGES().windows(2) {
        if let [a, b] = &pair {
            graph.entry(a.clone()).or_insert(vec![]).push(b.clone());
        }
    }

    for edge_list in edges {
        for pair in edge_list.windows(2) {
            if let [a, b] = &pair {
                graph.entry(a.clone()).or_insert(vec![]).push(b.clone());
            }
        }
    }

    // event_id -> StateEvent
    let mut event_map: BTreeMap<EventId, Arc<StateEvent>> = BTreeMap::new();
    // event_id -> StateMap<EventId>
    let mut state_at_event: BTreeMap<EventId, StateMap<EventId>> = BTreeMap::new();

    // resolve the current state and add it to the state_at_event map then continue
    // on in "time"
    for node in
        StateResolution::lexicographical_topological_sort(&graph, |id| (0, UNIX_EPOCH, id.clone()))
    {
        let fake_event = fake_event_map.get(&node).unwrap();
        let event_id = fake_event.event_id().clone();

        let prev_events = graph.get(&node).unwrap();

        let state_before: StateMap<EventId> = if prev_events.is_empty() {
            BTreeMap::new()
        } else if prev_events.len() == 1 {
            state_at_event.get(&prev_events[0]).unwrap().clone()
        } else {
            let state_sets = prev_events
                .iter()
                .filter_map(|k| state_at_event.get(k))
                .cloned()
                .collect::<Vec<_>>();

            tracing::info!(
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
                Some(event_map.clone()),
                &store,
            );
            match resolved {
                Ok(state) => state,
                Err(e) => panic!("resolution for {} failed: {}", node, e),
            }
        };

        let mut state_after = state_before.clone();

        if fake_event.state_key().is_some() {
            let ty = fake_event.kind();
            let key = fake_event.state_key();
            state_after.insert((ty, key), event_id.clone());
        }

        let auth_types = state_res::auth_types_for_event(
            &fake_event.kind(),
            fake_event.sender(),
            fake_event.state_key(),
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
            e.state_key().as_deref(),
            e.content(),
            &auth_events,
            prev_events,
        );

        // This can be used to sort of test this function
        // match StateResolution::apply_event(
        //     &room_id(),
        //     &RoomVersionId::Version6,
        //     Arc::clone(&event),
        //     &state_after,
        //     Some(event_map.clone()),
        //     &store,
        // ) {
        //     Ok(res) => {
        //         println!(
        //             "res contains: {} passed: {} for {}\n{:?}",
        //             state_after
        //                 .get(&(event.kind, event.state_key()))
        //                 .map(|id| id == &ev_id)
        //                 .unwrap_or_default(),
        //             res,
        //             event.event_id.clone().as_str(),
        //             event
        //                 .prev_event_ids()
        //                 .iter()
        //                 .map(|id| id.to_string())
        //                 .collect::<Vec<_>>()
        //         );
        //     }
        //     Err(e) => panic!("resolution for {} failed: {}", node, e),
        // }

        // we have to update our store, an actual user of this lib would
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
            event_map
                .keys()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
        ));

        let key = (ev.kind(), ev.state_key());

        expected_state.insert(key, node);
    }

    let start_state = state_at_event.get(&event_id("$START:foo")).unwrap();

    let end_state = state_at_event
        .get(&event_id("$END:foo"))
        .unwrap()
        .iter()
        .filter(|(k, v)| expected_state.contains_key(k) || start_state.get(k) != Some(*v))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<StateMap<EventId>>();

    assert_eq!(expected_state, end_state);
}

pub struct TestStore<E: Event>(pub BTreeMap<EventId, Arc<E>>);

#[allow(unused)]
impl<E: Event> StateStore<E> for TestStore<E> {
    fn get_event(&self, room_id: &RoomId, event_id: &EventId) -> Result<Arc<E>> {
        self.0
            .get(event_id)
            .map(Arc::clone)
            .ok_or_else(|| Error::NotFound(format!("{} not found", event_id.to_string())))
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
    serde_json::to_value(MemberEventContent {
        membership: MembershipState::Ban,
        displayname: None,
        avatar_url: None,
        is_direct: None,
        third_party_invite: None,
    })
    .unwrap()
}

pub fn member_content_join() -> JsonValue {
    serde_json::to_value(MemberEventContent {
        membership: MembershipState::Join,
        displayname: None,
        avatar_url: None,
        is_direct: None,
        third_party_invite: None,
    })
    .unwrap()
}

pub fn to_init_pdu_event(
    id: &str,
    sender: UserId,
    ev_type: EventType,
    state_key: Option<&str>,
    content: JsonValue,
) -> Arc<StateEvent> {
    let ts = unsafe {
        let ts = SERVER_TIMESTAMP;
        // increment the "origin_server_ts" value
        SERVER_TIMESTAMP += 1;
        ts
    };
    let id = if id.contains('$') {
        id.to_string()
    } else {
        format!("${}:foo", id)
    };

    let json = if let Some(state_key) = state_key {
        json!({
            "auth_events": [],
            "prev_events": [],
            "event_id": id,
            "sender": sender,
            "type": ev_type,
            "state_key": state_key,
            "content": content,
            "origin_server_ts": ts,
            "room_id": room_id(),
            "origin": "foo",
            "depth": 0,
            "hashes": { "sha256": "hello" },
            "signatures": {},
        })
    } else {
        json!({
            "auth_events": [],
            "prev_events": [],
            "event_id": id,
            "sender": sender,
            "type": ev_type,
            "content": content,
            "origin_server_ts": ts,
            "room_id": room_id(),
            "origin": "foo",
            "depth": 0,
            "hashes": { "sha256": "hello" },
            "signatures": {},
        })
    };
    Arc::new(serde_json::from_value(json).unwrap())
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
    let ts = unsafe {
        let ts = SERVER_TIMESTAMP;
        // increment the "origin_server_ts" value
        SERVER_TIMESTAMP += 1;
        ts
    };
    let id = if id.contains('$') {
        id.to_string()
    } else {
        format!("${}:foo", id)
    };
    let auth_events = auth_events
        .iter()
        .map(AsRef::as_ref)
        .map(event_id)
        .collect::<Vec<_>>();
    let prev_events = prev_events
        .iter()
        .map(AsRef::as_ref)
        .map(event_id)
        .collect::<Vec<_>>();

    let json = if let Some(state_key) = state_key {
        json!({
            "auth_events": auth_events,
            "prev_events": prev_events,
            "event_id": id,
            "sender": sender,
            "type": ev_type,
            "state_key": state_key,
            "content": content,
            "origin_server_ts": ts,
            "room_id": room_id(),
            "origin": "foo",
            "depth": 0,
            "hashes": { "sha256": "hello" },
            "signatures": {},
        })
    } else {
        json!({
            "auth_events": auth_events,
            "prev_events": prev_events,
            "event_id": id,
            "sender": sender,
            "type": ev_type,
            "content": content,
            "origin_server_ts": ts,
            "room_id": room_id(),
            "origin": "foo",
            "depth": 0,
            "hashes": { "sha256": "hello" },
            "signatures": {},
        })
    };
    Arc::new(serde_json::from_value(json).unwrap())
}

// all graphs start with these input events
#[allow(non_snake_case)]
pub fn INITIAL_EVENTS() -> BTreeMap<EventId, Arc<StateEvent>> {
    // this is always called so we can init the logger here
    let _ = LOGGER.call_once(|| {
        tracer::fmt()
            .with_env_filter(tracer::EnvFilter::from_default_env())
            .init()
    });

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
            json!({"users": {alice().to_string(): 100}}),
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
            None,
            json!({}),
            &[],
            &[],
        ),
        to_pdu_event::<EventId>(
            "END",
            charlie(),
            EventType::RoomMessage,
            None,
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
    use std::{collections::BTreeMap, time::SystemTime};

    use ruma::{
        events::{
            from_raw_json_value,
            pdu::{EventHash, Pdu},
            room::member::{MemberEventContent, MembershipState},
            EventDeHelper, EventType,
        },
        serde::CanonicalJsonValue,
        signatures::reference_hash,
        EventId, RoomId, RoomVersionId, ServerName, UInt, UserId,
    };
    use serde::{de, ser, Deserialize, Serialize};
    use serde_json::{value::RawValue as RawJsonValue, Value as JsonValue};

    use state_res::Event;

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
        fn origin_server_ts(&self) -> SystemTime {
            *self.origin_server_ts()
        }

        fn state_key(&self) -> Option<String> {
            self.state_key()
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
        fn signatures(&self) -> BTreeMap<Box<ServerName>, BTreeMap<ruma::ServerSigningKeyId, String>> {
            self.signatures()
        }
        fn unsigned(&self) -> &BTreeMap<String, JsonValue> {
            self.unsigned()
        }
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct EventIdHelper {
        event_id: EventId,
    }

    /// This feature is turned on in conduit but off when the tests run because
    /// we rely on the EventId to check the state-res.
    #[cfg(feature = "gen-eventid")]
    fn event_id<E: de::Error>(json: &RawJsonValue) -> Result<EventId, E> {
        use std::convert::TryFrom;
        EventId::try_from(format!(
            "${}",
            reference_hash(&from_raw_json_value(&json)?, &RoomVersionId::Version6)
                .map_err(de::Error::custom)?,
        ))
        .map_err(de::Error::custom)
    }

    /// Only turned on for testing where we need to keep the ID.
    #[cfg(not(feature = "gen-eventid"))]
    fn event_id<E: de::Error>(json: &RawJsonValue) -> Result<EventId, E> {
        use std::convert::TryFrom;
        Ok(match from_raw_json_value::<EventIdHelper, E>(&json) {
            Ok(id) => id.event_id,
            Err(_) => {
                // panic!("NOT DURING TESTS");
                EventId::try_from(format!(
                    "${}",
                    reference_hash(&from_raw_json_value(&json)?, &RoomVersionId::Version6)
                        .map_err(de::Error::custom)?,
                ))
                .map_err(de::Error::custom)?
            }
        })
    }

    // TODO: This no longer needs to be an enum now that PduStub is gone
    #[derive(Clone, Debug)]
    pub enum StateEvent {
        Full(EventId, Pdu),
    }

    impl Serialize for StateEvent {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
        {
            use ser::Error;
            use std::convert::TryInto;

            match self {
                Self::Full(id, ev) => {
                    // TODO: do we want to add the eventId when we
                    // serialize
                    let val: CanonicalJsonValue = serde_json::to_value(ev)
                        .map_err(S::Error::custom)?
                        .try_into()
                        .map_err(S::Error::custom)?;

                    match val {
                        CanonicalJsonValue::Object(mut obj) => {
                            obj.insert(
                                "event_id".into(),
                                ruma::serde::to_canonical_value(id).map_err(S::Error::custom)?,
                            );
                            obj.serialize(serializer)
                        }
                        _ => panic!("Pdu not an object"),
                    }
                }
            }
        }
    }

    impl<'de> de::Deserialize<'de> for StateEvent {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            let json = Box::<RawJsonValue>::deserialize(deserializer)?;
            let EventDeHelper {
                room_id, unsigned, ..
            } = from_raw_json_value(&json)?;

            // TODO: do we even want to try for the existing ID

            // Determine whether the event is a full or stub
            // based on the fields present.
            Ok(if room_id.is_some() {
                match unsigned {
                    Some(unsigned) if unsigned.redacted_because.is_some() => {
                        panic!("TODO deal with redacted events")
                    }
                    _ => StateEvent::Full(
                        event_id(&json)?,
                        Pdu::RoomV3Pdu(from_raw_json_value(&json)?),
                    ),
                }
            } else {
                panic!("Found stub event")
            })
        }
    }

    impl StateEvent {
        pub fn from_id_value(id: EventId, json: serde_json::Value) -> Result<Self, serde_json::Error> {
            Ok(Self::Full(
                id,
                Pdu::RoomV3Pdu(serde_json::from_value(json)?),
            ))
        }

        pub fn from_id_canon_obj(
            id: EventId,
            json: ruma::serde::CanonicalJsonObject,
        ) -> Result<Self, serde_json::Error> {
            Ok(Self::Full(
                id,
                // TODO: this is unfortunate (from_value(to_value(json)))...
                Pdu::RoomV3Pdu(serde_json::from_value(serde_json::to_value(json)?)?),
            ))
        }

        pub fn is_power_event(&self) -> bool {
            match self {
                Self::Full(_, any_event) => match any_event {
                    Pdu::RoomV1Pdu(event) => match event.kind {
                        EventType::RoomPowerLevels
                        | EventType::RoomJoinRules
                        | EventType::RoomCreate => event.state_key == Some("".into()),
                        EventType::RoomMember => {
                            if let Ok(content) =
                                // TODO fix clone
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
                },
            }
        }
        pub fn deserialize_content<C: serde::de::DeserializeOwned>(
            &self,
        ) -> Result<C, serde_json::Error> {
            match self {
                Self::Full(_, ev) => match ev {
                    Pdu::RoomV1Pdu(ev) => serde_json::from_value(ev.content.clone()),
                    Pdu::RoomV3Pdu(ev) => serde_json::from_value(ev.content.clone()),
                },
            }
        }
        pub fn origin_server_ts(&self) -> &SystemTime {
            match self {
                Self::Full(_, ev) => match ev {
                    Pdu::RoomV1Pdu(ev) => &ev.origin_server_ts,
                    Pdu::RoomV3Pdu(ev) => &ev.origin_server_ts,
                },
            }
        }
        pub fn event_id(&self) -> &EventId {
            match self {
                // TODO; make this a &EventId
                Self::Full(id, _) => id,
            }
        }

        pub fn sender(&self) -> &UserId {
            match self {
                Self::Full(_, ev) => match ev {
                    Pdu::RoomV1Pdu(ev) => &ev.sender,
                    Pdu::RoomV3Pdu(ev) => &ev.sender,
                },
            }
        }

        pub fn redacts(&self) -> Option<&EventId> {
            match self {
                Self::Full(_, ev) => match ev {
                    Pdu::RoomV1Pdu(ev) => ev.redacts.as_ref(),
                    Pdu::RoomV3Pdu(ev) => ev.redacts.as_ref(),
                },
            }
        }

        pub fn room_id(&self) -> &RoomId {
            match self {
                Self::Full(_, ev) => match ev {
                    Pdu::RoomV1Pdu(ev) => &ev.room_id,
                    Pdu::RoomV3Pdu(ev) => &ev.room_id,
                },
            }
        }
        pub fn kind(&self) -> EventType {
            match self {
                Self::Full(_, ev) => match ev {
                    Pdu::RoomV1Pdu(ev) => ev.kind.clone(),
                    Pdu::RoomV3Pdu(ev) => ev.kind.clone(),
                },
            }
        }
        pub fn state_key(&self) -> Option<String> {
            match self {
                Self::Full(_, ev) => match ev {
                    Pdu::RoomV1Pdu(ev) => ev.state_key.clone(),
                    Pdu::RoomV3Pdu(ev) => ev.state_key.clone(),
                },
            }
        }

        #[cfg(not(feature = "unstable-pre-spec"))]
        pub fn origin(&self) -> String {
            match self {
                Self::Full(_, ev) => match ev {
                    Pdu::RoomV1Pdu(ev) => ev.origin.clone(),
                    Pdu::RoomV3Pdu(ev) => ev.origin.clone(),
                },
            }
        }

        pub fn prev_event_ids(&self) -> Vec<EventId> {
            match self {
                Self::Full(_, ev) => match ev {
                    Pdu::RoomV1Pdu(ev) => ev.prev_events.iter().map(|(id, _)| id).cloned().collect(),
                    Pdu::RoomV3Pdu(ev) => ev.prev_events.clone(),
                },
            }
        }

        pub fn auth_events(&self) -> Vec<EventId> {
            match self {
                Self::Full(_, ev) => match ev {
                    Pdu::RoomV1Pdu(ev) => ev.auth_events.iter().map(|(id, _)| id).cloned().collect(),
                    Pdu::RoomV3Pdu(ev) => ev.auth_events.to_vec(),
                },
            }
        }

        pub fn content(&self) -> serde_json::Value {
            match self {
                Self::Full(_, ev) => match ev {
                    Pdu::RoomV1Pdu(ev) => ev.content.clone(),
                    Pdu::RoomV3Pdu(ev) => ev.content.clone(),
                },
            }
        }

        pub fn unsigned(&self) -> &BTreeMap<String, serde_json::Value> {
            match self {
                Self::Full(_, ev) => match ev {
                    Pdu::RoomV1Pdu(ev) => &ev.unsigned,
                    Pdu::RoomV3Pdu(ev) => &ev.unsigned,
                },
            }
        }

        pub fn signatures(
            &self,
        ) -> BTreeMap<Box<ServerName>, BTreeMap<ruma::ServerSigningKeyId, String>> {
            match self {
                Self::Full(_, ev) => match ev {
                    Pdu::RoomV1Pdu(_) => maplit::btreemap! {},
                    Pdu::RoomV3Pdu(ev) => ev.signatures.clone(),
                },
            }
        }

        pub fn hashes(&self) -> &EventHash {
            match self {
                Self::Full(_, ev) => match ev {
                    Pdu::RoomV1Pdu(ev) => &ev.hashes,
                    Pdu::RoomV3Pdu(ev) => &ev.hashes,
                },
            }
        }

        pub fn depth(&self) -> &UInt {
            match self {
                Self::Full(_, ev) => match ev {
                    Pdu::RoomV1Pdu(ev) => &ev.depth,
                    Pdu::RoomV3Pdu(ev) => &ev.depth,
                },
            }
        }

        pub fn is_type_and_key(&self, ev_type: EventType, state_key: &str) -> bool {
            match self {
                Self::Full(_, ev) => match ev {
                    Pdu::RoomV1Pdu(ev) => {
                        ev.kind == ev_type && ev.state_key.as_deref() == Some(state_key)
                    }
                    Pdu::RoomV3Pdu(ev) => {
                        ev.kind == ev_type && ev.state_key.as_deref() == Some(state_key)
                    }
                },
            }
        }

        /// Returns the room version this event is formatted for.
        ///
        /// Currently either version 1 or 6 is returned, 6 represents
        /// version 3 and above.
        pub fn room_version(&self) -> RoomVersionId {
            // TODO: We have to know the actual room version this is not sufficient
            match self {
                Self::Full(_, ev) => match ev {
                    Pdu::RoomV1Pdu(_) => RoomVersionId::Version1,
                    Pdu::RoomV3Pdu(_) => RoomVersionId::Version6,
                },
            }
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn deserialize_pdu() {
            let non_canonical_json = r#"{"auth_events": ["$FEKmyWTamMqoL3zkEC3mVPg3qkcXcUShxxaq5BltsCE", "$Oc8MYrZ3-eM4yBbhlj8YkYYluF9KHFDKU5uDpO-Ewcc", "$3ImCSXY6bbWbZ5S2N6BMplHHlP7RkxWZCM9fMbdM2NY", "$8Lfs0rVCE9bHQrUztEF9kbsrT4zASnPEtpImZN4L2n8"], "content": {"membership": "join"}, "depth": 135, "hashes": {"sha256": "Q7OehFJaB32W3NINZKesQZH7+ba7xZVFuyCtuWQ2emk"}, "origin": "pc.koesters.xyz:59003", "origin_server_ts": 1599901756522, "prev_events": ["$Oc8MYrZ3-eM4yBbhlj8YkYYluF9KHFDKU5uDpO-Ewcc"], "prev_state": [], "room_id": "!eGNyCFvnKcpsnIZiEV:koesters.xyz", "sender": "@timo:pc.koesters.xyz:59003", "state_key": "@timo:pc.koesters.xyz:59003", "type": "m.room.member", "signatures": {"koesters.xyz": {"ed25519:a_wwQy": "bb8T5haywaEXKNxUUjeNBfjYi/Qe32R/dGliduIs3Ct913WGzXYLjWh7xHqapie7viHPzkDw/KYJacpAYKvMBA"}, "pc.koesters.xyz:59003": {"ed25519:key1": "/B3tpaMZKoLNITrup4fbFhbIMWixxEKM49nS4MiKOFfyJjDGuC5nWsurw0m2eYzrffhkF5qQQ8+RlFvkqwqkBw"}}, "unsigned": {"age": 30, "replaces_state": "$Oc8MYrZ3-eM4yBbhlj8YkYYluF9KHFDKU5uDpO-Ewcc", "prev_content": {"membership": "join"}, "prev_sender": "@timo:pc.koesters.xyz:59003"}}"#;

            let pdu = serde_json::from_str::<StateEvent>(non_canonical_json).unwrap();

            assert_eq!(
                match &pdu {
                    StateEvent::Full(id, _) => id,
                },
                &ruma::event_id!("$Sfx_o8eLfo4idpTO8_IGrKSPKoRMC1CmQugVw9tu_MU")
            );
        }

        #[test]
        fn serialize_pdu() {
            let non_canonical_json = r#"{"auth_events": ["$FEKmyWTamMqoL3zkEC3mVPg3qkcXcUShxxaq5BltsCE", "$Oc8MYrZ3-eM4yBbhlj8YkYYluF9KHFDKU5uDpO-Ewcc", "$3ImCSXY6bbWbZ5S2N6BMplHHlP7RkxWZCM9fMbdM2NY", "$8Lfs0rVCE9bHQrUztEF9kbsrT4zASnPEtpImZN4L2n8"], "content": {"membership": "join"}, "depth": 135, "hashes": {"sha256": "Q7OehFJaB32W3NINZKesQZH7+ba7xZVFuyCtuWQ2emk"}, "origin": "pc.koesters.xyz:59003", "origin_server_ts": 1599901756522, "prev_events": ["$Oc8MYrZ3-eM4yBbhlj8YkYYluF9KHFDKU5uDpO-Ewcc"], "prev_state": [], "room_id": "!eGNyCFvnKcpsnIZiEV:koesters.xyz", "sender": "@timo:pc.koesters.xyz:59003", "state_key": "@timo:pc.koesters.xyz:59003", "type": "m.room.member", "signatures": {"koesters.xyz": {"ed25519:a_wwQy": "bb8T5haywaEXKNxUUjeNBfjYi/Qe32R/dGliduIs3Ct913WGzXYLjWh7xHqapie7viHPzkDw/KYJacpAYKvMBA"}, "pc.koesters.xyz:59003": {"ed25519:key1": "/B3tpaMZKoLNITrup4fbFhbIMWixxEKM49nS4MiKOFfyJjDGuC5nWsurw0m2eYzrffhkF5qQQ8+RlFvkqwqkBw"}}, "unsigned": {"age": 30, "replaces_state": "$Oc8MYrZ3-eM4yBbhlj8YkYYluF9KHFDKU5uDpO-Ewcc", "prev_content": {"membership": "join"}, "prev_sender": "@timo:pc.koesters.xyz:59003"}}"#;

            let pdu = serde_json::from_str::<StateEvent>(non_canonical_json).unwrap();

            assert_eq!(
                match &pdu {
                    StateEvent::Full(id, _) => id,
                },
                &ruma::event_id!("$Sfx_o8eLfo4idpTO8_IGrKSPKoRMC1CmQugVw9tu_MU")
            );

            // TODO: the `origin` field is left out, though it seems it should be part of the eventId hashing
            // For testing we must serialize the PDU with the `event_id` field this is probably not correct for production
            // although without them we get "Invalid bytes in DB" errors in conduit
            assert_eq!(
                ruma::serde::to_canonical_json_string(&pdu).unwrap(),
                r#"{"auth_events":["$FEKmyWTamMqoL3zkEC3mVPg3qkcXcUShxxaq5BltsCE","$Oc8MYrZ3-eM4yBbhlj8YkYYluF9KHFDKU5uDpO-Ewcc","$3ImCSXY6bbWbZ5S2N6BMplHHlP7RkxWZCM9fMbdM2NY","$8Lfs0rVCE9bHQrUztEF9kbsrT4zASnPEtpImZN4L2n8"],"content":{"membership":"join"},"depth":135,"event_id":"$Sfx_o8eLfo4idpTO8_IGrKSPKoRMC1CmQugVw9tu_MU","hashes":{"sha256":"Q7OehFJaB32W3NINZKesQZH7+ba7xZVFuyCtuWQ2emk"},"origin_server_ts":1599901756522,"prev_events":["$Oc8MYrZ3-eM4yBbhlj8YkYYluF9KHFDKU5uDpO-Ewcc"],"room_id":"!eGNyCFvnKcpsnIZiEV:koesters.xyz","sender":"@timo:pc.koesters.xyz:59003","signatures":{"koesters.xyz":{"ed25519:a_wwQy":"bb8T5haywaEXKNxUUjeNBfjYi/Qe32R/dGliduIs3Ct913WGzXYLjWh7xHqapie7viHPzkDw/KYJacpAYKvMBA"},"pc.koesters.xyz:59003":{"ed25519:key1":"/B3tpaMZKoLNITrup4fbFhbIMWixxEKM49nS4MiKOFfyJjDGuC5nWsurw0m2eYzrffhkF5qQQ8+RlFvkqwqkBw"}},"state_key":"@timo:pc.koesters.xyz:59003","type":"m.room.member","unsigned":{"age":30,"prev_content":{"membership":"join"},"prev_sender":"@timo:pc.koesters.xyz:59003","replaces_state":"$Oc8MYrZ3-eM4yBbhlj8YkYYluF9KHFDKU5uDpO-Ewcc"}}"#,
            )
        }
    }

}