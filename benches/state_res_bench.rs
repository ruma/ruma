// Because of criterion `cargo bench` works,
// but if you use `cargo bench -- --save-baseline <name>`
// or pass any other args to it, it fails with the error
// `cargo bench unknown option --save-baseline`.
// To pass args to criterion, use this form
// `cargo bench --bench <name of the bench> -- --save-baseline <name>`.
use std::{collections::BTreeMap, convert::TryFrom, sync::Arc, time::UNIX_EPOCH};

use criterion::{criterion_group, criterion_main, Criterion};
use maplit::btreemap;
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

static mut SERVER_TIMESTAMP: i32 = 0;

fn lexico_topo_sort(c: &mut Criterion) {
    c.bench_function("lexicographical topological sort", |b| {
        let graph = btreemap! {
            event_id("l") => vec![event_id("o")],
            event_id("m") => vec![event_id("n"), event_id("o")],
            event_id("n") => vec![event_id("o")],
            event_id("o") => vec![], // "o" has zero outgoing edges but 4 incoming edges
            event_id("p") => vec![event_id("o")],
        };
        b.iter(|| {
            let _ = StateResolution::lexicographical_topological_sort(&graph, |id| {
                (0, UNIX_EPOCH, id.clone())
            });
        })
    });
}

fn resolution_shallow_auth_chain(c: &mut Criterion) {
    c.bench_function("resolve state of 5 events one fork", |b| {
        let mut store = TestStore(btreemap! {});

        // build up the DAG
        let (state_at_bob, state_at_charlie, _) = store.set_up();

        b.iter(|| {
            let _resolved = match StateResolution::resolve(
                &room_id(),
                &RoomVersionId::Version6,
                &[state_at_bob.clone(), state_at_charlie.clone()],
                None,
                &store,
            ) {
                Ok(state) => state,
                Err(e) => panic!("{}", e),
            };
        })
    });
}

fn resolve_deeper_event_set(c: &mut Criterion) {
    c.bench_function("resolve state of 10 events 3 conflicting", |b| {
        let init = INITIAL_EVENTS();
        let ban = BAN_STATE_SET();

        let mut inner = init;
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
        .map(|ev| ((ev.kind(), ev.state_key()), ev.event_id().clone()))
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
        .map(|ev| ((ev.kind(), ev.state_key()), ev.event_id().clone()))
        .collect::<StateMap<_>>();

        b.iter(|| {
            let _resolved = match StateResolution::resolve(
                &room_id(),
                &RoomVersionId::Version6,
                &[state_set_a.clone(), state_set_b.clone()],
                Some(inner.clone()),
                &store,
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

impl TestStore<event::StateEvent> {
    pub fn set_up(&mut self) -> (StateMap<EventId>, StateMap<EventId>, StateMap<EventId>) {
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
        self.0
            .insert(alice_mem.event_id().clone(), Arc::clone(&alice_mem));

        let join_rules = to_pdu_event(
            "IJR",
            alice(),
            EventType::RoomJoinRules,
            Some(""),
            json!({ "join_rule": JoinRule::Public }),
            &[cre.clone(), alice_mem.event_id().clone()],
            &[alice_mem.event_id().clone()],
        );
        self.0
            .insert(join_rules.event_id().clone(), join_rules.clone());

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
        self.0
            .insert(charlie_mem.event_id().clone(), charlie_mem.clone());

        let state_at_bob = [&create_event, &alice_mem, &join_rules, &bob_mem]
            .iter()
            .map(|e| ((e.kind(), e.state_key()), e.event_id().clone()))
            .collect::<StateMap<_>>();

        let state_at_charlie = [&create_event, &alice_mem, &join_rules, &charlie_mem]
            .iter()
            .map(|e| ((e.kind(), e.state_key()), e.event_id().clone()))
            .collect::<StateMap<_>>();

        let expected = [
            &create_event,
            &alice_mem,
            &join_rules,
            &bob_mem,
            &charlie_mem,
        ]
        .iter()
        .map(|e| ((e.kind(), e.state_key()), e.event_id().clone()))
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
    serde_json::to_value(MemberEventContent {
        membership: MembershipState::Ban,
        displayname: None,
        avatar_url: None,
        is_direct: None,
        third_party_invite: None,
    })
    .unwrap()
}

fn member_content_join() -> JsonValue {
    serde_json::to_value(MemberEventContent {
        membership: MembershipState::Join,
        displayname: None,
        avatar_url: None,
        is_direct: None,
        third_party_invite: None,
    })
    .unwrap()
}

fn to_pdu_event<S>(
    id: &str,
    sender: UserId,
    ev_type: EventType,
    state_key: Option<&str>,
    content: JsonValue,
    auth_events: &[S],
    prev_events: &[S],
) -> Arc<event::StateEvent>
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
fn INITIAL_EVENTS() -> BTreeMap<EventId, Arc<event::StateEvent>> {
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
fn BAN_STATE_SET() -> BTreeMap<EventId, Arc<event::StateEvent>> {
    vec![
        to_pdu_event(
            "PA",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({"users": {alice(): 100, bob(): 50}}),
            &["CREATE", "IMA", "IPOWER"], // auth_events
            &["START"],                   // prev_events
        ),
        to_pdu_event(
            "PB",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({"users": {alice(): 100, bob(): 50}}),
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
}