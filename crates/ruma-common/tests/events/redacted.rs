use assert_matches::assert_matches;
use js_int::uint;
use ruma_common::{
    event_id,
    events::{
        room::{
            aliases::RedactedRoomAliasesEventContent,
            create::{RedactedRoomCreateEventContent, RoomCreateEventContent},
            message::{RedactedRoomMessageEventContent, RoomMessageEventContent},
            redaction::{
                OriginalSyncRoomRedactionEvent, RoomRedactionEventContent, SyncRoomRedactionEvent,
            },
        },
        AnyMessageLikeEvent, AnyRoomEvent, AnySyncMessageLikeEvent, AnySyncRoomEvent,
        AnySyncStateEvent, EventContent, MessageLikeEvent, MessageLikeUnsigned, RedactContent,
        RedactedMessageLikeEvent, RedactedSyncMessageLikeEvent, RedactedSyncStateEvent,
        RedactedUnsigned, SyncMessageLikeEvent, SyncStateEvent,
    },
    room_id, server_name, user_id, MilliSecondsSinceUnixEpoch, RoomVersionId,
};
use serde_json::{
    from_value as from_json_value, json, to_value as to_json_value,
    value::to_raw_value as to_raw_json_value,
};

fn unsigned() -> RedactedUnsigned {
    let mut unsigned = RedactedUnsigned::default();
    unsigned.redacted_because =
        Some(Box::new(SyncRoomRedactionEvent::Original(OriginalSyncRoomRedactionEvent {
            content: RoomRedactionEventContent::with_reason("redacted because".into()),
            redacts: event_id!("$h29iv0s8:example.com").to_owned(),
            event_id: event_id!("$h29iv0s8:example.com").to_owned(),
            origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
            sender: user_id!("@carl:example.com").to_owned(),
            unsigned: MessageLikeUnsigned::default(),
        })));

    unsigned
}

#[test]
fn redacted_message_event_serialize() {
    let redacted = RedactedSyncMessageLikeEvent {
        content: RedactedRoomMessageEventContent::new(),
        event_id: event_id!("$h29iv0s8:example.com").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
        sender: user_id!("@carl:example.com").to_owned(),
        unsigned: RedactedUnsigned::default(),
    };

    let expected = json!({
      "event_id": "$h29iv0s8:example.com",
      "origin_server_ts": 1,
      "sender": "@carl:example.com",
      "type": "m.room.message",
    });

    let actual = to_json_value(&redacted).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn redacted_aliases_event_serialize_no_content() {
    let redacted = RedactedSyncStateEvent {
        content: RedactedRoomAliasesEventContent::default(),
        event_id: event_id!("$h29iv0s8:example.com").to_owned(),
        state_key: server_name!("example.com").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
        sender: user_id!("@carl:example.com").to_owned(),
        unsigned: RedactedUnsigned::default(),
    };

    let expected = json!({
      "event_id": "$h29iv0s8:example.com",
      "state_key": "example.com",
      "origin_server_ts": 1,
      "sender": "@carl:example.com",
      "type": "m.room.aliases",
    });

    let actual = to_json_value(&redacted).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn redacted_aliases_event_serialize_with_content() {
    let redacted = RedactedSyncStateEvent {
        content: RedactedRoomAliasesEventContent::new_v1(vec![]),
        event_id: event_id!("$h29iv0s8:example.com").to_owned(),
        state_key: server_name!("example.com").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
        sender: user_id!("@carl:example.com").to_owned(),
        unsigned: RedactedUnsigned::default(),
    };

    let expected = json!({
      "content": {
          "aliases": []
      },
      "event_id": "$h29iv0s8:example.com",
      "state_key": "example.com",
      "origin_server_ts": 1,
      "sender": "@carl:example.com",
      "type": "m.room.aliases",
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
      "type": "m.room.aliases",
    });

    let actual = to_json_value(&redacted).unwrap();

    assert_matches!(
        from_json_value::<AnySyncRoomEvent>(actual).unwrap(),
        AnySyncRoomEvent::State(AnySyncStateEvent::RoomAliases(
            SyncStateEvent::Redacted(RedactedSyncStateEvent {
                content: RedactedRoomAliasesEventContent { aliases, .. },
                event_id,
                ..
            }),
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
      "type": "m.room.message",
    });

    let actual = to_json_value(&redacted).unwrap();

    assert_matches!(
        from_json_value::<AnyRoomEvent>(actual).unwrap(),
        AnyRoomEvent::MessageLike(AnyMessageLikeEvent::RoomMessage(
            MessageLikeEvent::Redacted(RedactedMessageLikeEvent {
                content: RedactedRoomMessageEventContent { .. },
                event_id, room_id, ..
            }),
        )) if event_id == event_id!("$h29iv0s8:example.com")
            && room_id == room_id!("!roomid:room.com")
    )
}

#[test]
fn redacted_deserialize_any_room_sync() {
    let mut unsigned = RedactedUnsigned::default();
    // The presence of `redacted_because` triggers the event enum (AnySyncRoomEvent in this case)
    // to return early with `RedactedContent` instead of failing to deserialize according
    // to the event type string.
    unsigned.redacted_because =
        Some(Box::new(SyncRoomRedactionEvent::Original(OriginalSyncRoomRedactionEvent {
            content: RoomRedactionEventContent::with_reason("redacted because".into()),
            redacts: event_id!("$h29iv0s8:example.com").to_owned(),
            event_id: event_id!("$h29iv0s8:example.com").to_owned(),
            origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
            sender: user_id!("@carl:example.com").to_owned(),
            unsigned: MessageLikeUnsigned::default(),
        })));

    let redacted = json!({
      "event_id": "$h29iv0s8:example.com",
      "origin_server_ts": 1,
      "sender": "@carl:example.com",
      "unsigned": unsigned,
      "type": "m.room.message",
    });

    let actual = to_json_value(&redacted).unwrap();

    assert_matches!(
        from_json_value::<AnySyncRoomEvent>(actual).unwrap(),
        AnySyncRoomEvent::MessageLike(AnySyncMessageLikeEvent::RoomMessage(
            SyncMessageLikeEvent::Redacted(RedactedSyncMessageLikeEvent {
                content: RedactedRoomMessageEventContent { .. },
                event_id,
                ..
            }),
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
      "state_key": "",
      "unsigned": unsigned(),
      "type": "m.room.create",
    });

    assert_matches!(
        from_json_value::<AnySyncRoomEvent>(redacted)
            .unwrap(),
        AnySyncRoomEvent::State(AnySyncStateEvent::RoomCreate(
            SyncStateEvent::Redacted(RedactedSyncStateEvent {
                content: RedactedRoomCreateEventContent {
                    creator, ..
                },
                event_id,
                unsigned,
                ..
            }),
        )) if event_id == event_id!("$h29iv0s8:example.com")
            && unsigned.redacted_because.is_some()
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
        "type": "m.made.up",
    });

    assert_matches!(
        from_json_value::<AnySyncRoomEvent>(redacted.clone()),
        Ok(AnySyncRoomEvent::State(_))
    );

    let x = from_json_value::<AnySyncStateEvent>(redacted).unwrap();
    assert_eq!(x.event_id(), event_id!("$h29iv0s8:example.com"))
}

/* #[test]
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
        },
    });

    let redaction = OriginalSyncRoomRedactionEvent {
        content: RoomRedactionEventContent::with_reason("redacted because".into()),
        redacts: event_id!("$143273582443PhrSn:example.com").to_owned(),
        event_id: event_id!("$h29iv0s8:example.com").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
        sender: user_id!("@carl:example.com").to_owned(),
        unsigned: MessageLikeUnsigned::default(),
    };

    let event: AnyMessageLikeEvent = from_json_value(ev).unwrap();

    assert_matches!(
        event.redact(redaction, &RoomVersionId::V6),
        AnyMessageLikeEvent::RoomMessage(MessageLikeEvent::Redacted(RedactedMessageLikeEvent {
            content: RedactedRoomMessageEventContent { .. },
            event_id,
            room_id,
            sender,
            origin_server_ts,
            unsigned,
        })) if event_id == event_id!("$143273582443PhrSn:example.com")
            && unsigned.redacted_because.is_some()
            && room_id == room_id!("!roomid:room.com")
            && sender == user_id!("@user:example.com")
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(1))
    );
} */

#[test]
fn redact_message_content() {
    let json = json!({
        "body": "test",
        "msgtype": "m.audio",
        "url": "mxc://example.com/AuDi0",
    });

    let raw_json = to_raw_json_value(&json).unwrap();
    let content = RoomMessageEventContent::from_parts("m.room.message", &raw_json).unwrap();

    assert_matches!(content.redact(&RoomVersionId::V6), RedactedRoomMessageEventContent { .. });
}

#[test]
fn redact_state_content() {
    let json = json!({
        "creator": "@carl:example.com",
        "m.federate": true,
        "room_version": "4",
    });

    let raw_json = to_raw_json_value(&json).unwrap();
    let content = RoomCreateEventContent::from_parts("m.room.create", &raw_json).unwrap();

    assert_matches!(
        content.redact(&RoomVersionId::V6),
        RedactedRoomCreateEventContent {
            creator,
            ..
        } if creator == user_id!("@carl:example.com")
    );
}
