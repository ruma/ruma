use std::{
    convert::TryFrom,
    time::{Duration, UNIX_EPOCH},
};

use maplit::btreemap;
use matches::assert_matches;
use ruma_events::{
    room::redaction::{RedactedContent, RedactionEvent, RedactionEventContent},
    AnyMessageEvent, AnyMessageEventStub, AnyRoomEvent, AnyRoomEventStub, EventJson, MessageEvent,
    MessageEventStub, UnsignedData,
};
use ruma_identifiers::{EventId, RoomId, UserId};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn redacted_serialize() {
    let left_over_keys = btreemap! {
        "None".to_string() => json!("of message events keys survive"),
    };
    let redacted = MessageEventStub {
        content: RedactedContent {
            event_type: "m.room.message".to_string(),
            reason: Some("who cares".to_string()),
            left_over_keys,
        },
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        unsigned: UnsignedData::default(),
    };

    let expected = json!({
      "content": {
        "None": "of message events keys survive",
        "reason": "who cares"
      },
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
      "content": {
        "None": "of message events keys survive",
        "reason": "who cares"
      },
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
        AnyRoomEvent::Message(AnyMessageEvent::IsRedacted(MessageEvent {
            event_id, content, room_id, ..
        })) if event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
            && room_id == RoomId::try_from("!roomid:room.com").unwrap()
            && content.reason == Some("who cares".to_string())
            && content.event_type == "m.room.message"
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
      "content": {
        "None": "of message events keys survive",
        "reason": "who cares"
      },
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
        AnyRoomEventStub::Message(AnyMessageEventStub::IsRedacted(MessageEventStub {
            event_id, content, ..
        })) if event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
            && content.reason == Some("who cares".to_string())
            && content.event_type == "m.room.message"
    )
}

#[test]
fn redacted_deserialize_to_event_kind() {
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
        "None": "of message events keys survive",
        "reason": "who cares"
      },
      "event_id": "$h29iv0s8:example.com",
      "origin_server_ts": 1,
      "sender": "@carl:example.com",
      "unsigned": unsigned,
      "type": "m.room.message"
    });

    assert_matches!(
        from_json_value::<EventJson<MessageEventStub<RedactedContent>>>(redacted)
            .unwrap()
            .deserialize()
            .unwrap(),
        MessageEventStub {
            event_id, content, ..
        } if event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
            && content.reason == Some("who cares".to_string())
            && content.event_type == "m.room.message"
    )
}
