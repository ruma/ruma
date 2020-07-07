use std::{
    convert::TryFrom,
    time::{Duration, UNIX_EPOCH},
};

use matches::assert_matches;
use ruma_events::{
    room::{
        create::RedactedCreateEventContent,
        redaction::{RedactionEvent, RedactionEventContent},
    },
    AnyRedactedMessageEvent, AnyRedactedMessageEventStub, AnyRedactedStateEventStub, AnyRoomEvent,
    AnyRoomEventStub, EmptyRedactedMessageEvent, EmptyRedactedMessageEventStub, EventJson,
    RedactedStateEventStub, UnsignedData,
};
use ruma_identifiers::{EventId, RoomId, UserId};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn redacted_message_event_serialize() {
    let redacted = EmptyRedactedMessageEventStub {
        event_type: "m.room.message".to_string(),
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        unsigned: UnsignedData::default(),
    };

    let expected = json!({
      "event_id": "$h29iv0s8:example.com",
      "origin_server_ts": 1,
      "sender": "@carl:example.com",
      "type": "m.room.message"
    });

    let actual = to_json_value(&redacted).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn redacted_deserialize_any_room() {
    let mut unsigned = UnsignedData::default();
    // The presence of `redacted_because` triggers the event enum (AnyRoomEvent in this case)
    // to return early with `RedactedContent` instead of failing to deserialize itself according
    // to the event type string.
    unsigned.redacted_because = Some(EventJson::from(RedactionEvent {
        content: RedactionEventContent { reason: Some("redacted because".into()) },
        redacts: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        room_id: RoomId::try_from("!roomid:room.com").unwrap(),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        unsigned: UnsignedData::default(),
    }));

    let redacted = json!({
      "event_id": "$h29iv0s8:example.com",
      "room_id": "!roomid:room.com",
      "origin_server_ts": 1,
      "sender": "@carl:example.com",
      "unsigned": unsigned,
      "type": "m.room.message"
    });

    let actual = to_json_value(&redacted).unwrap();

    assert_matches!(
        from_json_value::<EventJson<AnyRoomEvent>>(actual)
            .unwrap()
            .deserialize()
            .unwrap(),
        AnyRoomEvent::RedactedMessage(AnyRedactedMessageEvent::RoomMessage(EmptyRedactedMessageEvent {
            event_id, room_id, event_type, ..
        })) if event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
            && room_id == RoomId::try_from("!roomid:room.com").unwrap()
            && event_type == "m.room.message"
    )
}

#[test]
fn redacted_deserialize_any_room_stub() {
    let mut unsigned = UnsignedData::default();
    // The presence of `redacted_because` triggers the event enum (AnyRoomEventStub in this case)
    // to return early with `RedactedContent` instead of failing to deserialize itself according
    // to the event type string.
    unsigned.redacted_because = Some(EventJson::from(RedactionEvent {
        content: RedactionEventContent { reason: Some("redacted because".into()) },
        redacts: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        room_id: RoomId::try_from("!roomid:room.com").unwrap(),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        unsigned: UnsignedData::default(),
    }));

    let redacted = json!({
      "event_id": "$h29iv0s8:example.com",
      "origin_server_ts": 1,
      "sender": "@carl:example.com",
      "unsigned": unsigned,
      "type": "m.room.message"
    });

    let actual = to_json_value(&redacted).unwrap();

    assert_matches!(
        from_json_value::<EventJson<AnyRoomEventStub>>(actual)
            .unwrap()
            .deserialize()
            .unwrap(),
        AnyRoomEventStub::RedactedMessage(AnyRedactedMessageEventStub::RoomMessage(EmptyRedactedMessageEventStub {
            event_id, event_type, ..
        })) if event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
            && event_type == "m.room.message"
    )
}

#[test]
fn redacted_state_event_deserialize() {
    let mut unsigned = UnsignedData::default();
    unsigned.redacted_because = Some(EventJson::from(RedactionEvent {
        content: RedactionEventContent { reason: Some("redacted because".into()) },
        redacts: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        room_id: RoomId::try_from("!roomid:room.com").unwrap(),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        unsigned: UnsignedData::default(),
    }));

    let redacted = json!({
      "content": {
        "creator": "@carl:example.com",
      },
      "event_id": "$h29iv0s8:example.com",
      "origin_server_ts": 1,
      "sender": "@carl:example.com",
      "state_key": "hello there",
      "unsigned": unsigned,
      "type": "m.room.create"
    });

    assert_matches!(
        from_json_value::<EventJson<AnyRoomEventStub>>(redacted)
            .unwrap()
            .deserialize()
            .unwrap(),
        AnyRoomEventStub::RedactedState(AnyRedactedStateEventStub::RoomCreate(RedactedStateEventStub {
            content: RedactedCreateEventContent {
                creator,
            },
            event_id, state_key, unsigned, ..
        })) if event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
            && unsigned.redacted_because.is_some()
            && state_key == "hello there"
            && creator == UserId::try_from("@carl:example.com").unwrap()
    )
}
