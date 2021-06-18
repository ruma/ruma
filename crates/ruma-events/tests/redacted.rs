use js_int::uint;
use matches::assert_matches;
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events::{
    custom::RedactedCustomEventContent,
    room::{
        aliases::RedactedAliasesEventContent,
        create::RedactedCreateEventContent,
        message::RedactedMessageEventContent,
        redaction::{RedactionEventContent, SyncRedactionEvent},
    },
    AnyMessageEvent, AnyMessageEventContent, AnyRedactedMessageEvent,
    AnyRedactedMessageEventContent, AnyRedactedStateEventContent, AnyRedactedSyncMessageEvent,
    AnyRedactedSyncStateEvent, AnyRoomEvent, AnyStateEventContent, AnySyncRoomEvent, EventContent,
    Redact, RedactContent, RedactedMessageEvent, RedactedSyncMessageEvent, RedactedSyncStateEvent,
    RedactedUnsigned, Unsigned,
};
use ruma_identifiers::{event_id, room_id, user_id, RoomVersionId};
use ruma_serde::Raw;
use serde_json::{
    from_value as from_json_value, json, to_value as to_json_value, value::to_raw_value,
};

fn unsigned() -> RedactedUnsigned {
    let mut unsigned = RedactedUnsigned::default();
    unsigned.redacted_because = Some(Box::new(SyncRedactionEvent {
        content: RedactionEventContent::with_reason("redacted because".into()),
        redacts: event_id!("$h29iv0s8:example.com"),
        event_id: event_id!("$h29iv0s8:example.com"),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
        sender: user_id!("@carl:example.com"),
        unsigned: Unsigned::default(),
    }));

    unsigned
}

#[test]
fn redacted_message_event_serialize() {
    let redacted = RedactedSyncMessageEvent {
        content: RedactedMessageEventContent::new(),
        event_id: event_id!("$h29iv0s8:example.com"),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
        sender: user_id!("@carl:example.com"),
        unsigned: RedactedUnsigned::default(),
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
        content: RedactedAliasesEventContent::default(),
        event_id: event_id!("$h29iv0s8:example.com"),
        state_key: "".into(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
        sender: user_id!("@carl:example.com"),
        unsigned: RedactedUnsigned::default(),
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
        content: RedactedAliasesEventContent::new_v1(vec![]),
        event_id: event_id!("$h29iv0s8:example.com"),
        state_key: "".to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
        sender: user_id!("@carl:example.com"),
        unsigned: RedactedUnsigned::default(),
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
    let redacted = json!({
      "event_id": "$h29iv0s8:example.com",
      "origin_server_ts": 1,
      "sender": "@carl:example.com",
      "state_key": "hello",
      "unsigned": unsigned(),
      "type": "m.room.aliases"
    });

    let actual = to_json_value(&redacted).unwrap();

    assert_matches!(
        from_json_value::<Raw<AnySyncRoomEvent>>(actual)
            .unwrap()
            .deserialize()
            .unwrap(),
        AnySyncRoomEvent::RedactedState(AnyRedactedSyncStateEvent::RoomAliases(
            RedactedSyncStateEvent {
                content: RedactedAliasesEventContent { aliases, .. },
                event_id,
                ..
            },
        )) if event_id == event_id!("$h29iv0s8:example.com")
            && aliases.is_none()
    )
}

#[test]
fn redacted_deserialize_any_room() {
    let redacted = json!({
      "event_id": "$h29iv0s8:example.com",
      "room_id": "!roomid:room.com",
      "origin_server_ts": 1,
      "sender": "@carl:example.com",
      "unsigned": unsigned(),
      "type": "m.room.message"
    });

    let actual = to_json_value(&redacted).unwrap();

    assert_matches!(
        from_json_value::<Raw<AnyRoomEvent>>(actual)
            .unwrap()
            .deserialize()
            .unwrap(),
        AnyRoomEvent::RedactedMessage(AnyRedactedMessageEvent::RoomMessage(RedactedMessageEvent {
            content: RedactedMessageEventContent { .. },
            event_id, room_id, ..
        })) if event_id == event_id!("$h29iv0s8:example.com")
            && room_id == room_id!("!roomid:room.com")
    )
}

#[test]
fn redacted_deserialize_any_room_sync() {
    let mut unsigned = RedactedUnsigned::default();
    // The presence of `redacted_because` triggers the event enum (AnySyncRoomEvent in this case)
    // to return early with `RedactedContent` instead of failing to deserialize according
    // to the event type string.
    unsigned.redacted_because = Some(Box::new(SyncRedactionEvent {
        content: RedactionEventContent::with_reason("redacted because".into()),
        redacts: event_id!("$h29iv0s8:example.com"),
        event_id: event_id!("$h29iv0s8:example.com"),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
        sender: user_id!("@carl:example.com"),
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
        AnySyncRoomEvent::RedactedMessage(AnyRedactedSyncMessageEvent::RoomMessage(
            RedactedSyncMessageEvent {
                content: RedactedMessageEventContent { .. },
                event_id,
                ..
            }
        )) if event_id == event_id!("$h29iv0s8:example.com")
    )
}

#[test]
fn redacted_state_event_deserialize() {
    let redacted = json!({
      "content": {
        "creator": "@carl:example.com",
      },
      "event_id": "$h29iv0s8:example.com",
      "origin_server_ts": 1,
      "sender": "@carl:example.com",
      "state_key": "hello there",
      "unsigned": unsigned(),
      "type": "m.room.create"
    });

    assert_matches!(
        from_json_value::<Raw<AnySyncRoomEvent>>(redacted)
            .unwrap()
            .deserialize()
            .unwrap(),
        AnySyncRoomEvent::RedactedState(AnyRedactedSyncStateEvent::RoomCreate(
            RedactedSyncStateEvent {
                content: RedactedCreateEventContent {
                    creator, ..
                },
                event_id,
                state_key,
                unsigned,
                ..
            }
        )) if event_id == event_id!("$h29iv0s8:example.com")
            && unsigned.redacted_because.is_some()
            && state_key == "hello there"
            && creator == user_id!("@carl:example.com")
    )
}

#[test]
fn redacted_custom_event_serialize() {
    let redacted = json!({
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "sender": "@carl:example.com",
        "state_key": "hello there",
        "unsigned": unsigned(),
        "type": "m.made.up"
    });

    assert_matches!(
        from_json_value::<AnySyncRoomEvent>(redacted.clone()),
        Ok(AnySyncRoomEvent::RedactedState(_))
    );

    let x = from_json_value::<AnyRedactedSyncStateEvent>(redacted).unwrap();
    assert_eq!(x.event_id(), &event_id!("$h29iv0s8:example.com"))
}

#[test]
fn redacted_custom_event_deserialize() {
    let unsigned = unsigned();

    let redacted = RedactedSyncStateEvent {
        content: RedactedCustomEventContent { event_type: "m.made.up".into() },
        event_id: event_id!("$h29iv0s8:example.com"),
        sender: user_id!("@carl:example.com"),
        state_key: "hello there".into(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
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
            "url": "mxc://example.com/AuDi0",
        }
    });

    let redaction = SyncRedactionEvent {
        content: RedactionEventContent::with_reason("redacted because".into()),
        redacts: event_id!("$143273582443PhrSn:example.com"),
        event_id: event_id!("$h29iv0s8:example.com"),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
        sender: user_id!("@carl:example.com"),
        unsigned: Unsigned::default(),
    };

    let event = from_json_value::<Raw<AnyMessageEvent>>(ev).unwrap().deserialize().unwrap();

    assert_matches!(
        event.redact(redaction, &RoomVersionId::Version6),
        AnyRedactedMessageEvent::RoomMessage(RedactedMessageEvent {
            content: RedactedMessageEventContent { .. },
            event_id,
            room_id,
            sender,
            origin_server_ts,
            unsigned,
        }) if event_id == event_id!("$143273582443PhrSn:example.com")
            && unsigned.redacted_because.is_some()
            && room_id == room_id!("!roomid:room.com")
            && sender == user_id!("@user:example.com")
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(1))
    );
}

#[test]
fn redact_message_content() {
    let json = json!({
        "body": "test",
        "msgtype": "m.audio",
        "url": "mxc://example.com/AuDi0",
    });

    let raw_json = to_raw_value(&json).unwrap();
    let content = AnyMessageEventContent::from_parts("m.room.message", &raw_json).unwrap();

    assert_matches!(
        content.redact(&RoomVersionId::Version6),
        AnyRedactedMessageEventContent::RoomMessage(RedactedMessageEventContent { .. })
    );
}

#[test]
fn redact_state_content() {
    let json = json!({
        "creator": "@carl:example.com",
        "m.federate": true,
        "room_version": "4"
    });

    let raw_json = to_raw_value(&json).unwrap();
    let content = AnyStateEventContent::from_parts("m.room.create", &raw_json).unwrap();

    assert_matches!(
        content.redact(&RoomVersionId::Version6),
        AnyRedactedStateEventContent::RoomCreate(RedactedCreateEventContent {
            creator,
            ..
        }) if creator == user_id!("@carl:example.com")
    );
}
