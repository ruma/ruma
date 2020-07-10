use std::{
    convert::TryFrom,
    time::{Duration, UNIX_EPOCH},
};

use matches::assert_matches;
use ruma_events::{
    custom::RedactedCustomEventContent,
    room::{
        aliases::RedactedAliasesEventContent,
        create::RedactedCreateEventContent,
        message::RedactedMessageEventContent,
        redaction::{RedactionEvent, RedactionEventContent},
    },
    AnyRedactedMessageEvent, AnyRedactedMessageEventStub, AnyRedactedStateEventStub, AnyRoomEvent,
    AnyRoomEventStub, EventJson, RedactedMessageEvent, RedactedMessageEventStub,
    RedactedStateEventStub, UnsignedData,
};
use ruma_identifiers::{EventId, RoomId, UserId};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

fn is_zst<T>(_: &T) -> bool {
    std::mem::size_of::<T>() == 0
}

#[test]
fn redacted_message_event_serialize() {
    let redacted = RedactedMessageEventStub {
        content: RedactedMessageEventContent,
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
fn redacted_aliases_event_serialize() {
    let redacted = RedactedStateEventStub {
        content: RedactedAliasesEventContent { aliases: None },
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        state_key: "".to_string(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        unsigned: UnsignedData::default(),
    };

    let expected = json!({
      "event_id": "$h29iv0s8:example.com",
      "state_key": "",
      "origin_server_ts": 1,
      "sender": "@carl:example.com",
      "type": "m.room.aliases"
    });

    let actual = to_json_value(&redacted).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn redacted_deserialize_any_room() {
    let mut unsigned = UnsignedData::default();
    // The presence of `redacted_because` triggers the event enum (AnyRoomEvent in this case)
    // to return early with `RedactedContent` instead of failing to deserialize according
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
        AnyRoomEvent::RedactedMessage(AnyRedactedMessageEvent::RoomMessage(RedactedMessageEvent {
            event_id, room_id, content, ..
        })) if event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
            && room_id == RoomId::try_from("!roomid:room.com").unwrap()
            && is_zst(&content)
    )
}

#[test]
fn redacted_deserialize_any_room_stub() {
    let mut unsigned = UnsignedData::default();
    // The presence of `redacted_because` triggers the event enum (AnyRoomEventStub in this case)
    // to return early with `RedactedContent` instead of failing to deserialize according
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
        AnyRoomEventStub::RedactedMessage(AnyRedactedMessageEventStub::RoomMessage(RedactedMessageEventStub {
            event_id, content, ..
        })) if event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
            && is_zst(&content)
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

#[test]
fn redacted_custom_event_serialize() {
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
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "sender": "@carl:example.com",
        "state_key": "hello there",
        "unsigned": unsigned,
        "type": "m.made.up"
    });

    assert_matches!(
        from_json_value::<EventJson<AnyRoomEventStub>>(redacted.clone())
            .unwrap()
            .deserialize()
            .unwrap(),
        AnyRoomEventStub::RedactedState(AnyRedactedStateEventStub::Custom(RedactedStateEventStub {
            content: RedactedCustomEventContent {
                event_type,
            },
            event_id, state_key, unsigned, ..
        })) if event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
            && unsigned.redacted_because.is_some()
            && state_key == "hello there"
            && event_type == "m.made.up"
    );

    let x = from_json_value::<EventJson<crate::AnyRedactedStateEventStub>>(redacted)
        .unwrap()
        .deserialize()
        .unwrap();
    assert_eq!(x.event_id(), &EventId::try_from("$h29iv0s8:example.com").unwrap())
}

#[test]
fn redacted_custom_event_deserialize() {
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

    let redacted = RedactedStateEventStub {
        content: RedactedCustomEventContent { event_type: "m.made.up".to_string() },
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        state_key: "hello there".to_string(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        unsigned: unsigned.clone(),
    };

    let expected = json!({
      "event_id": "$h29iv0s8:example.com",
      "origin_server_ts": 1,
      "sender": "@carl:example.com",
      "state_key": "hello there",
      "unsigned": unsigned,
      "type": "m.made.up"
    });

    let actual = to_json_value(&redacted).unwrap();
    assert_eq!(actual, expected);
}
