use std::{
    collections::{BTreeMap, HashMap},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering::SeqCst},
    },
};

use js_int::uint;
use ruma_common::{
    EventId, MilliSecondsSinceUnixEpoch, OwnedEventId, RoomId, ServerSignatures, UserId, room_id,
    user_id,
};
use ruma_events::{
    StateEventType, TimelineEventType,
    room::{
        join_rules::{JoinRule, RoomJoinRulesEventContent},
        member::{MembershipState, RoomMemberEventContent},
    },
};
use serde_json::{
    json,
    value::{RawValue as RawJsonValue, to_raw_value as to_raw_json_value},
};

pub(crate) use self::event::{EventHash, PduEvent};
use crate::{Event, events::RoomCreateEvent};

static SERVER_TIMESTAMP: AtomicU64 = AtomicU64::new(0);

pub(crate) fn event_id(id: &str) -> OwnedEventId {
    if id.contains('$') {
        return id.try_into().unwrap();
    }

    format!("${id}:foo").try_into().unwrap()
}

pub(crate) fn alice() -> &'static UserId {
    user_id!("@alice:foo")
}

pub(crate) fn bob() -> &'static UserId {
    user_id!("@bob:foo")
}

pub(crate) fn charlie() -> &'static UserId {
    user_id!("@charlie:foo")
}

pub(crate) fn ella() -> &'static UserId {
    user_id!("@ella:foo")
}

pub(crate) fn zara() -> &'static UserId {
    user_id!("@zara:foo")
}

pub(crate) fn room_id() -> &'static RoomId {
    room_id!("!test:foo")
}

pub(crate) fn v12_room_id() -> &'static RoomId {
    room_id!("!CREATE")
}

pub(crate) fn member_content_ban() -> Box<RawJsonValue> {
    to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Ban)).unwrap()
}

pub(crate) fn member_content_join() -> Box<RawJsonValue> {
    to_raw_json_value(&RoomMemberEventContent::new(MembershipState::Join)).unwrap()
}

pub(crate) fn to_init_pdu_event(
    id: &str,
    sender: &UserId,
    ev_type: TimelineEventType,
    state_key: Option<&str>,
    content: Box<RawJsonValue>,
) -> Arc<PduEvent> {
    let ts = SERVER_TIMESTAMP.fetch_add(1, SeqCst);
    let id = if id.contains('$') { id.to_owned() } else { format!("${id}:foo") };

    let state_key = state_key.map(ToOwned::to_owned);
    Arc::new(PduEvent {
        event_id: id.try_into().unwrap(),
        room_id: Some(room_id().to_owned()),
        sender: sender.to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(ts.try_into().unwrap()),
        state_key,
        kind: ev_type,
        content,
        redacts: None,
        unsigned: BTreeMap::new(),
        auth_events: vec![],
        prev_events: vec![],
        depth: uint!(0),
        hashes: EventHash { sha256: "".to_owned() },
        signatures: ServerSignatures::default(),
        rejected: false,
    })
}

pub(crate) fn to_pdu_event<S>(
    id: &str,
    sender: &UserId,
    ev_type: TimelineEventType,
    state_key: Option<&str>,
    content: Box<RawJsonValue>,
    auth_events: &[S],
    prev_events: &[S],
) -> Arc<PduEvent>
where
    S: AsRef<str>,
{
    let ts = SERVER_TIMESTAMP.fetch_add(1, SeqCst);
    let id = if id.contains('$') { id.to_owned() } else { format!("${id}:foo") };
    let auth_events = auth_events.iter().map(AsRef::as_ref).map(event_id).collect::<Vec<_>>();
    let prev_events = prev_events.iter().map(AsRef::as_ref).map(event_id).collect::<Vec<_>>();

    let state_key = state_key.map(ToOwned::to_owned);
    Arc::new(PduEvent {
        event_id: id.try_into().unwrap(),
        room_id: Some(room_id().to_owned()),
        sender: sender.to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(ts.try_into().unwrap()),
        state_key,
        kind: ev_type,
        content,
        redacts: None,
        unsigned: BTreeMap::new(),
        auth_events,
        prev_events,
        depth: uint!(0),
        hashes: EventHash { sha256: "".to_owned() },
        signatures: ServerSignatures::default(),
        rejected: false,
    })
}

/// Same as `to_pdu_event()`, but uses the default m.room.create event ID to generate the room ID.
pub(crate) fn to_v12_pdu_event<S>(
    id: &str,
    sender: &UserId,
    ev_type: TimelineEventType,
    state_key: Option<&str>,
    content: Box<RawJsonValue>,
    auth_events: &[S],
    prev_events: &[S],
) -> Arc<PduEvent>
where
    S: AsRef<str>,
{
    fn event_id(id: &str) -> OwnedEventId {
        if id.contains('$') { id.try_into().unwrap() } else { format!("${id}").try_into().unwrap() }
    }

    let ts = SERVER_TIMESTAMP.fetch_add(1, SeqCst);
    let auth_events = auth_events.iter().map(AsRef::as_ref).map(event_id).collect::<Vec<_>>();
    let prev_events = prev_events.iter().map(AsRef::as_ref).map(event_id).collect::<Vec<_>>();

    let state_key = state_key.map(ToOwned::to_owned);
    Arc::new(PduEvent {
        event_id: event_id(id),
        room_id: Some(v12_room_id().to_owned()),
        sender: sender.to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(ts.try_into().unwrap()),
        state_key,
        kind: ev_type,
        content,
        redacts: None,
        unsigned: BTreeMap::new(),
        auth_events,
        prev_events,
        depth: uint!(0),
        hashes: EventHash { sha256: "".to_owned() },
        signatures: ServerSignatures::default(),
        rejected: false,
    })
}

pub(crate) fn room_redaction_pdu_event<S>(
    id: &str,
    sender: &UserId,
    redacts: OwnedEventId,
    content: Box<RawJsonValue>,
    auth_events: &[S],
    prev_events: &[S],
) -> Arc<PduEvent>
where
    S: AsRef<str>,
{
    let ts = SERVER_TIMESTAMP.fetch_add(1, SeqCst);
    let id = if id.contains('$') { id.to_owned() } else { format!("${id}:foo") };
    let auth_events = auth_events.iter().map(AsRef::as_ref).map(event_id).collect::<Vec<_>>();
    let prev_events = prev_events.iter().map(AsRef::as_ref).map(event_id).collect::<Vec<_>>();

    Arc::new(PduEvent {
        event_id: id.try_into().unwrap(),
        room_id: Some(room_id().to_owned()),
        sender: sender.to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(ts.try_into().unwrap()),
        state_key: None,
        kind: TimelineEventType::RoomRedaction,
        content,
        redacts: Some(redacts),
        unsigned: BTreeMap::new(),
        auth_events,
        prev_events,
        depth: uint!(0),
        hashes: EventHash { sha256: "".to_owned() },
        signatures: ServerSignatures::default(),
        rejected: false,
    })
}

pub(crate) fn room_create_v12_pdu_event(
    id: &str,
    sender: &UserId,
    content: Box<RawJsonValue>,
) -> Arc<PduEvent> {
    let ts = SERVER_TIMESTAMP.fetch_add(1, SeqCst);
    let id = if id.contains('$') { id.to_owned() } else { format!("${id}") };

    Arc::new(PduEvent {
        event_id: id.try_into().unwrap(),
        room_id: None,
        sender: sender.to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(ts.try_into().unwrap()),
        state_key: Some(String::new()),
        kind: TimelineEventType::RoomCreate,
        content,
        redacts: None,
        unsigned: BTreeMap::new(),
        auth_events: vec![],
        prev_events: vec![],
        depth: uint!(0),
        hashes: EventHash { sha256: "".to_owned() },
        signatures: ServerSignatures::default(),
        rejected: false,
    })
}

/// Batch of initial events to use for incoming events in the v1-v11 room versions.
#[allow(non_snake_case)]
pub(crate) fn INITIAL_EVENTS() -> HashMap<OwnedEventId, Arc<PduEvent>> {
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
            Some(bob().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "IPOWER"],
            &["IJR"],
        ),
        to_pdu_event(
            "IMC",
            charlie(),
            TimelineEventType::RoomMember,
            Some(charlie().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "IPOWER"],
            &["IMB"],
        ),
        to_pdu_event::<&EventId>(
            "START",
            charlie(),
            TimelineEventType::RoomMessage,
            Some("dummy"),
            to_raw_json_value(&json!({})).unwrap(),
            &[],
            &[],
        ),
        to_pdu_event::<&EventId>(
            "END",
            charlie(),
            TimelineEventType::RoomMessage,
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

/// Batch of initial events to use for incoming events from room version 12 onwards.
#[allow(non_snake_case)]
pub(crate) fn INITIAL_V12_EVENTS() -> HashMap<OwnedEventId, Arc<PduEvent>> {
    vec![
        room_create_v12_pdu_event(
            "CREATE",
            alice(),
            to_raw_json_value(&json!({ "room_version": "12" })).unwrap(),
        ),
        to_v12_pdu_event(
            "IMA",
            alice(),
            TimelineEventType::RoomMember,
            Some(alice().as_str()),
            member_content_join(),
            &["CREATE"],
            &["CREATE"],
        ),
        to_v12_pdu_event(
            "IPOWER",
            alice(),
            TimelineEventType::RoomPowerLevels,
            Some(""),
            to_raw_json_value(&json!({})).unwrap(),
            &["CREATE", "IMA"],
            &["IMA"],
        ),
        to_v12_pdu_event(
            "IJR",
            alice(),
            TimelineEventType::RoomJoinRules,
            Some(""),
            to_raw_json_value(&RoomJoinRulesEventContent::new(JoinRule::Public)).unwrap(),
            &["CREATE", "IMA", "IPOWER"],
            &["IPOWER"],
        ),
        to_v12_pdu_event(
            "IMB",
            bob(),
            TimelineEventType::RoomMember,
            Some(bob().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "IPOWER"],
            &["IJR"],
        ),
        to_v12_pdu_event(
            "IMC",
            charlie(),
            TimelineEventType::RoomMember,
            Some(charlie().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "IPOWER"],
            &["IMB"],
        ),
        to_v12_pdu_event::<&EventId>(
            "START",
            charlie(),
            TimelineEventType::RoomMessage,
            Some("dummy"),
            to_raw_json_value(&json!({})).unwrap(),
            &[],
            &[],
        ),
        to_v12_pdu_event::<&EventId>(
            "END",
            charlie(),
            TimelineEventType::RoomMessage,
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
pub(crate) fn INITIAL_EVENTS_CREATE_ROOM() -> HashMap<OwnedEventId, Arc<PduEvent>> {
    vec![to_pdu_event::<&EventId>(
        "CREATE",
        alice(),
        TimelineEventType::RoomCreate,
        Some(""),
        to_raw_json_value(&json!({ "creator": alice() })).unwrap(),
        &[],
        &[],
    )]
    .into_iter()
    .map(|ev| (ev.event_id().to_owned(), ev))
    .collect()
}

pub(crate) mod event {
    use std::collections::BTreeMap;

    use js_int::UInt;
    use ruma_common::{
        MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId, OwnedUserId, RoomId,
        ServerSignatures, UserId,
    };
    use ruma_events::TimelineEventType;
    use serde::{Deserialize, Serialize};
    use serde_json::value::RawValue as RawJsonValue;

    use crate::Event;

    impl Event for PduEvent {
        type Id = OwnedEventId;

        fn event_id(&self) -> &Self::Id {
            &self.event_id
        }

        fn room_id(&self) -> Option<&RoomId> {
            self.room_id.as_deref()
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
        pub(crate) event_id: OwnedEventId,

        /// The room this event belongs to.
        pub(crate) room_id: Option<OwnedRoomId>,

        /// The user id of the user who sent this event.
        pub(crate) sender: OwnedUserId,

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
        pub(crate) prev_events: Vec<OwnedEventId>,

        /// The maximum depth of the `prev_events`, plus one.
        pub(crate) depth: UInt,

        /// Event IDs for the authorization events that would allow this event to be
        /// in the room.
        pub(crate) auth_events: Vec<OwnedEventId>,

        /// For redaction events, the ID of the event being redacted.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) redacts: Option<OwnedEventId>,

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

/// Wrapper around a state map.
pub(crate) struct TestStateMap(HashMap<StateEventType, HashMap<String, Arc<PduEvent>>>);

impl TestStateMap {
    /// Construct a `TestStateMap` from the given event map.
    pub(crate) fn new(events: &HashMap<OwnedEventId, Arc<PduEvent>>) -> Self {
        let mut state_map: HashMap<StateEventType, HashMap<String, Arc<PduEvent>>> = HashMap::new();

        for event in events.values() {
            let event_type = StateEventType::from(event.event_type().to_string());

            state_map
                .entry(event_type)
                .or_default()
                .insert(event.state_key().unwrap().to_owned(), event.clone());
        }

        TestStateMap(state_map)
    }

    /// Get the event with the given event type and state key.
    pub(crate) fn get(
        &self,
        event_type: &StateEventType,
        state_key: &str,
    ) -> Option<&Arc<PduEvent>> {
        self.0.get(event_type)?.get(state_key)
    }

    /// A function to get a state event from this map.
    pub(crate) fn fetch_state_fn<'a>(
        &'a self,
    ) -> impl Fn(&StateEventType, &str) -> Option<&'a Arc<PduEvent>> + Copy {
        |event_type: &StateEventType, state_key: &str| self.get(event_type, state_key)
    }

    /// The `m.room.create` event contained in this map.
    ///
    /// Panics if there is no `m.room.create` event in this map.
    pub(crate) fn room_create_event(&self) -> RoomCreateEvent<&Arc<PduEvent>> {
        RoomCreateEvent::new(self.get(&StateEventType::RoomCreate, "").unwrap())
    }
}

/// Create an `m.room.third_party_invite` event with the given sender.
pub(crate) fn room_third_party_invite(sender: &UserId) -> Arc<PduEvent> {
    let content = json!({
        "display_name": "o...@g...",
        "key_validity_url": "https://identity.local/_matrix/identity/v2/pubkey/isvalid",
        "public_key": "Gb9ECWmEzf6FQbrBZ9w7lshQhqowtrbLDFw4rXAxZuE",
        "public_keys": [
            {
                "key_validity_url": "https://identity.local/_matrix/identity/v2/pubkey/isvalid",
                "public_key": "Gb9ECWmEzf6FQbrBZ9w7lshQhqowtrbLDFw4rXAxZuE"
            },
            {
                "key_validity_url": "https://identity.local/_matrix/identity/v2/pubkey/ephemeral/isvalid",
                "public_key": "Kxdvv7lo0O6JVI7yimFgmYPfpLGnctcpYjuypP5zx/c"
            }
        ]
    });

    to_pdu_event(
        "THIRDPARTY",
        sender,
        TimelineEventType::RoomThirdPartyInvite,
        Some("somerandomtoken"),
        to_raw_json_value(&content).unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IPOWER"],
    )
}
