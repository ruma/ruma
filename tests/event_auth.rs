use std::{collections::BTreeMap, convert::TryFrom, sync::Arc};

use ruma::{
    events::{
        pdu::EventHash,
        room::{
            join_rules::JoinRule,
            member::{MemberEventContent, MembershipState},
        },
        EventType,
    },
    identifiers::{EventId, RoomId, UserId},
};
use serde_json::{json, Value as JsonValue};
#[rustfmt::skip] // this deletes the comments for some reason yay!
use state_res::{
    event_auth::{
        // auth_check, auth_types_for_event, can_federate, check_power_levels, check_redaction,
        valid_membership_change,
    },
    Requester, StateEvent, StateMap, StateStore, Result, Error
};
use tracing_subscriber as tracer;

use std::sync::Once;

static LOGGER: Once = Once::new();

static mut SERVER_TIMESTAMP: i32 = 0;

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

pub struct TestStore(BTreeMap<EventId, Arc<StateEvent>>);

#[allow(unused)]
impl StateStore for TestStore {
    fn get_event(&self, room_id: &RoomId, event_id: &EventId) -> Result<Arc<StateEvent>> {
        self.0
            .get(event_id)
            .map(Arc::clone)
            .ok_or_else(|| Error::NotFound(format!("{} not found", event_id.to_string())))
    }
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
        .map(|id| {
            (
                id,
                EventHash {
                    sha256: "hello".into(),
                },
            )
        })
        .collect::<Vec<_>>();
    let prev_events = prev_events
        .iter()
        .map(AsRef::as_ref)
        .map(event_id)
        .map(|id| {
            (
                id,
                EventHash {
                    sha256: "hello".into(),
                },
            )
        })
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
fn INITIAL_EVENTS() -> BTreeMap<EventId, Arc<StateEvent>> {
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
    ]
    .into_iter()
    .map(|ev| (ev.event_id(), ev))
    .collect()
}

#[test]
fn test_ban_pass() {
    let events = INITIAL_EVENTS();

    let prev = events
        .values()
        .find(|ev| ev.event_id().as_str().contains("IMC"))
        .map(Arc::clone);

    let auth_events = events
        .values()
        .map(|ev| ((ev.kind(), ev.state_key()), Arc::clone(ev)))
        .collect::<StateMap<_>>();

    let requester = Requester {
        prev_event_ids: vec![event_id("IMC")],
        room_id: &room_id(),
        content: &member_content_ban(),
        state_key: Some(charlie().to_string()),
        sender: &alice(),
    };

    assert!(valid_membership_change(requester, prev, None, &auth_events).unwrap())
}

#[test]
fn test_ban_fail() {
    let events = INITIAL_EVENTS();

    let prev = events
        .values()
        .find(|ev| ev.event_id().as_str().contains("IMC"))
        .map(Arc::clone);

    let auth_events = events
        .values()
        .map(|ev| ((ev.kind(), ev.state_key()), Arc::clone(ev)))
        .collect::<StateMap<_>>();

    let requester = Requester {
        prev_event_ids: vec![event_id("IMC")],
        room_id: &room_id(),
        content: &member_content_ban(),
        state_key: Some(alice().to_string()),
        sender: &charlie(),
    };

    assert!(!valid_membership_change(requester, prev, None, &auth_events).unwrap())
}
