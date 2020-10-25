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
    identifiers::{EventId, RoomId, RoomVersionId, UserId},
};
use serde_json::{json, Value as JsonValue};
use state_res::{Error, Result, StateEvent, StateMap, StateStore};
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
    .map(|ev| (ev.event_id(), ev))
    .collect()
}

fn shuffle(list: &mut [EventId]) {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    for i in 1..list.len() {
        let j = rng.gen_range(0, list.len());
        list.swap(i, j);
    }
}

fn test_event_sort() {
    let mut events = INITIAL_EVENTS();
    let store = TestStore(events.clone());

    let event_map = events
        .values()
        .map(|ev| ((ev.kind(), ev.state_key()), ev.clone()))
        .collect::<StateMap<_>>();

    let auth_chain = &[] as &[_];

    let power_events = event_map
        .values()
        .filter(|pdu| pdu.is_power_event())
        .map(|pdu| pdu.event_id())
        .collect::<Vec<_>>();

    // This is a TODO in conduit
    // TODO these events are not guaranteed to be sorted but they are resolved, do
    // we need the auth_chain
    let sorted_power_events = state_res::StateResolution::reverse_topological_power_sort(
        &room_id(),
        &power_events,
        &mut events,
        &store,
        &auth_chain,
    );

    // This is a TODO in conduit
    // TODO we may be able to skip this since they are resolved according to spec
    let resolved_power = state_res::StateResolution::iterative_auth_check(
        &room_id(),
        &RoomVersionId::Version6,
        &sorted_power_events,
        &BTreeMap::new(), // unconflicted events
        &mut events,
        &store,
    )
    .expect("iterative auth check failed on resolved events");

    // don't remove any events so we know it sorts them all correctly
    let mut events_to_sort = events.keys().cloned().collect::<Vec<_>>();

    shuffle(&mut events_to_sort);

    let power_level = resolved_power.get(&(EventType::RoomPowerLevels, "".into()));

    let sorted_event_ids = state_res::StateResolution::mainline_sort(
        &room_id(),
        &events_to_sort,
        power_level,
        &mut events,
        &store,
    );

    assert_eq!(
        vec![
            "$CREATE:foo",
            "$IMA:foo",
            "$IPOWER:foo",
            "$IJR:foo",
            "$IMB:foo",
            "$IMC:foo",
            "$END:foo"
        ],
        sorted_event_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
    )
}

#[test]
fn test_sort() {
    for _ in 0..20 {
        // since we shuffle the eventIds before we sort them introducing randomness
        // seems like we should test this a few times
        test_event_sort()
    }
}
