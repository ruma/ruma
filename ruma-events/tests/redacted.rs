use std::{
    convert::TryFrom,
    time::{Duration, UNIX_EPOCH},
};

use matches::assert_matches;
use ruma_common::Raw;
use ruma_events::{
    custom::RedactedCustomEventContent,
    room::{
        aliases::RedactedAliasesEventContent,
        create::RedactedCreateEventContent,
        message::RedactedMessageEventContent,
        redaction::{RedactionEvent, RedactionEventContent, SyncRedactionEvent},
    },
    AnyMessageEvent, AnyRedactedMessageEvent, AnyRedactedSyncMessageEvent,
    AnyRedactedSyncStateEvent, AnyRoomEvent, AnySyncRoomEvent, EventJson, RedactedMessageEvent,
    RedactedSyncMessageEvent, RedactedSyncStateEvent, RedactedSyncUnsigned, RedactedUnsigned,
    Unsigned,
};
use ruma_identifiers::{EventId, RoomId, RoomVersionId, UserId};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

fn full_unsigned() -> RedactedSyncUnsigned {
    let mut unsigned = RedactedSyncUnsigned::default();
    // The presence of `redacted_because` triggers the event enum to return early
    // with `RedactedContent` instead of failing to deserialize according
    // to the event type string.
    unsigned.redacted_because = Some(Raw::from(SyncRedactionEvent {
        content: RedactionEventContent { reason: Some("redacted because".into()) },
        redacts: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        unsigned: Unsigned::default(),
    }));

    unsigned
}

#[test]
fn redacted_message_event_serialize() {
    let redacted = RedactedSyncMessageEvent {
        content: RedactedMessageEventContent,
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        unsigned: RedactedSyncUnsigned::default(),
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
fn redacted_aliases_event_serialize_no_content() {
    let redacted = RedactedSyncStateEvent {
        content: RedactedAliasesEventContent { aliases: None },
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        state_key: "".into(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        unsigned: RedactedSyncUnsigned::default(),
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
fn redacted_aliases_event_serialize_with_content() {
    let redacted = RedactedSyncStateEvent {
        content: RedactedAliasesEventContent { aliases: Some(vec![]) },
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        state_key: "".to_string(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        unsigned: RedactedSyncUnsigned::default(),
    };

    let expected = json!({
      "content": {
          "aliases": []
      },
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
fn redacted_aliases_deserialize() {
    let unsigned = full_unsigned();

    let redacted = json!({
      "event_id": "$h29iv0s8:example.com",
      "origin_server_ts": 1,
      "sender": "@carl:example.com",
      "state_key": "hello",
      "unsigned": unsigned,
      "type": "m.room.aliases"
    });

    let actual = to_json_value(&redacted).unwrap();

    assert_matches!(
        from_json_value::<Raw<AnySyncRoomEvent>>(actual)
            .unwrap()
            .deserialize()
            .unwrap(),
        AnySyncRoomEvent::RedactedState(AnyRedactedSyncStateEvent::RoomAliases(RedactedSyncStateEvent {
            content: RedactedAliasesEventContent { aliases },
            event_id, ..
        })) if event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
            && aliases.is_none()
    )
}

#[test]
fn redacted_deserialize_any_room() {
    let unsigned = full_unsigned();

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
        from_json_value::<Raw<AnyRoomEvent>>(actual)
            .unwrap()
            .deserialize()
            .unwrap(),
        AnyRoomEvent::RedactedMessage(AnyRedactedMessageEvent::RoomMessage(RedactedMessageEvent {
            content: RedactedMessageEventContent,
            event_id, room_id, ..
        })) if event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
            && room_id == RoomId::try_from("!roomid:room.com").unwrap()
    )
}

#[test]
fn redacted_deserialize_any_room_sync() {
    let mut unsigned = RedactedUnsigned::default();
    // The presence of `redacted_because` triggers the event enum (AnySyncRoomEvent in this case)
    // to return early with `RedactedContent` instead of failing to deserialize according
    // to the event type string.
    unsigned.redacted_because = Some(Raw::from(RedactionEvent {
        content: RedactionEventContent { reason: Some("redacted because".into()) },
        redacts: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        room_id: RoomId::try_from("!roomid:room.com").unwrap(),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        unsigned: Unsigned::default(),
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
        from_json_value::<Raw<AnySyncRoomEvent>>(actual)
            .unwrap()
            .deserialize()
            .unwrap(),
        AnySyncRoomEvent::RedactedMessage(AnyRedactedSyncMessageEvent::RoomMessage(RedactedSyncMessageEvent {
            content: RedactedMessageEventContent,
            event_id, ..
        })) if event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
    )
}

#[test]
fn redacted_state_event_deserialize() {
    let unsigned = full_unsigned();

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
        from_json_value::<Raw<AnySyncRoomEvent>>(redacted)
            .unwrap()
            .deserialize()
            .unwrap(),
        AnySyncRoomEvent::RedactedState(AnyRedactedSyncStateEvent::RoomCreate(RedactedSyncStateEvent {
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
    let unsigned = full_unsigned();

    let redacted = json!({
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "sender": "@carl:example.com",
        "state_key": "hello there",
        "unsigned": unsigned,
        "type": "m.made.up"
    });

    assert_matches!(
        from_json_value::<Raw<AnySyncRoomEvent>>(redacted.clone())
            .unwrap()
            .deserialize()
            .unwrap(),
        AnySyncRoomEvent::RedactedState(AnyRedactedSyncStateEvent::Custom(RedactedSyncStateEvent {
            content: RedactedCustomEventContent {
                event_type,
            },
            event_id, state_key, unsigned, ..
        })) if event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
            && unsigned.redacted_because.is_some()
            && state_key == "hello there"
            && event_type == "m.made.up"
    );

    let x = from_json_value::<EventJson<AnyRedactedSyncStateEvent>>(redacted)
        .unwrap()
        .deserialize()
        .unwrap();
    assert_eq!(x.event_id(), &EventId::try_from("$h29iv0s8:example.com").unwrap())
}

#[test]
fn redacted_custom_event_deserialize() {
    let unsigned = full_unsigned();

    let redacted = RedactedSyncStateEvent {
        content: RedactedCustomEventContent { event_type: "m.made.up".into() },
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        state_key: "hello there".into(),
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

#[test]
fn redact_method_properly_redacts() {
    let ev = json!({
        "type": "m.room.message",
        "event_id": "$143273582443PhrSn:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "sender": "@user:example.com",
        "content": {
            "body": "test",
            "msgtype": "m.audio",
            "url": "http://example.com/audio.mp3",
        }
    });

    let redaction = RedactionEvent {
        content: RedactionEventContent { reason: Some("redacted because".into()) },
        redacts: EventId::try_from("$143273582443PhrSn:example.com").unwrap(),
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        room_id: RoomId::try_from("!roomid:room.com").unwrap(),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        unsigned: Unsigned::default(),
    };

    let event = from_json_value::<EventJson<AnyMessageEvent>>(ev).unwrap().deserialize().unwrap();

    assert_matches!(
        event.redact(redaction, RoomVersionId::version_6()),
        AnyRedactedMessageEvent::RoomMessage(RedactedMessageEvent {
            content: RedactedMessageEventContent,
            event_id,
            room_id,
            sender,
            origin_server_ts,
            unsigned,
        }) if event_id == EventId::try_from("$143273582443PhrSn:example.com").unwrap()
            && unsigned.redacted_because.is_some()
            && room_id == RoomId::try_from("!roomid:room.com").unwrap()
            && sender == UserId::try_from("@user:example.com").unwrap()
            && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
    );
}
