use assert_matches2::assert_matches;
use ruma_common::{
    canonical_json::assert_to_canonical_json_eq, room_version_rules::RedactionRules,
};
use ruma_events::{
    AnyMessageLikeEvent, AnySyncMessageLikeEvent, AnySyncStateEvent, AnySyncTimelineEvent,
    AnyTimelineEvent, EventContentFromType, MessageLikeEvent, RedactContent, SyncMessageLikeEvent,
    room::{
        aliases::RoomAliasesEventContent,
        create::RoomCreateEventContent,
        message::{RedactedRoomMessageEventContent, RoomMessageEventContent},
        redaction::RoomRedactionEventContent,
    },
};
use serde_json::{
    Value as JsonValue, from_value as from_json_value, json,
    value::to_raw_value as to_raw_json_value,
};

fn unsigned() -> JsonValue {
    json!({
        "redacted_because": {
            "type": "m.room.redaction",
            "content": RoomRedactionEventContent::new_v1().with_reason("redacted because".into()),
            "redacts": "$h29iv0s8:example.com",
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "sender": "@carl:example.com",
        }
    })
}

#[test]
fn serialize_redacted_message_event_content() {
    assert_to_canonical_json_eq!(RedactedRoomMessageEventContent::new(), json!({}));
}

#[test]
fn serialize_empty_redacted_aliases_event_content() {
    let mut content = RoomAliasesEventContent::new(vec![]);
    content.aliases.take();
    assert_to_canonical_json_eq!(content, json!({}));
}

#[test]
fn redacted_aliases_event_serialize_with_content() {
    assert_to_canonical_json_eq!(RoomAliasesEventContent::new(vec![]), json!({ "aliases": [] }),);
}

#[test]
fn deserialize_redacted_state() {
    let redacted = json!({
        "content": {},
        "event_id": "$h29iv0s8:example.com",
        "room_id": "!roomid:room.com",
        "origin_server_ts": 1,
        "sender": "@carl:example.com",
        "state_key": "hello",
        "unsigned": unsigned(),
        "type": "m.room.aliases",
    });

    assert_matches!(
        from_json_value::<AnySyncTimelineEvent>(redacted),
        Ok(AnySyncTimelineEvent::State(event))
    );

    assert_matches!(event, AnySyncStateEvent::RoomAliases(redacted));
    assert_eq!(redacted.event_id, "$h29iv0s8:example.com");
    assert_eq!(redacted.content.aliases, None);
}

#[test]
fn deserialize_redacted_message_like() {
    let redacted = json!({
        "content": {},
        "event_id": "$h29iv0s8:example.com",
        "room_id": "!roomid:room.com",
        "origin_server_ts": 1,
        "sender": "@carl:example.com",
        "unsigned": unsigned(),
        "type": "m.room.message",
    });

    assert_matches!(
        from_json_value::<AnyTimelineEvent>(redacted),
        Ok(AnyTimelineEvent::MessageLike(event))
    );
    assert!(event.is_redacted());

    assert_matches!(event, AnyMessageLikeEvent::RoomMessage(MessageLikeEvent::Redacted(redacted)));
    assert_eq!(redacted.event_id, "$h29iv0s8:example.com");
    assert_eq!(redacted.room_id, "!roomid:room.com");
}

#[test]
fn deserialize_redacted_sync_message_like() {
    let redacted = json!({
        "content": {},
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "sender": "@carl:example.com",
        "unsigned": unsigned(),
        "type": "m.room.message",
    });

    assert_matches!(
        from_json_value::<AnySyncTimelineEvent>(redacted),
        Ok(AnySyncTimelineEvent::MessageLike(event))
    );
    assert!(event.is_redacted());

    assert_matches!(
        event,
        AnySyncMessageLikeEvent::RoomMessage(SyncMessageLikeEvent::Redacted(redacted))
    );
    assert_eq!(redacted.event_id, "$h29iv0s8:example.com");
}

#[test]
#[allow(deprecated)]
fn deserialize_redacted_sync_state() {
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
        from_json_value::<AnySyncTimelineEvent>(redacted),
        Ok(AnySyncTimelineEvent::State(event))
    );

    assert_matches!(event, AnySyncStateEvent::RoomCreate(redacted));
    assert_eq!(redacted.event_id, "$h29iv0s8:example.com");
    assert_eq!(redacted.content.creator.unwrap(), "@carl:example.com");
}

#[test]
fn deserialize_redacted_custom_sync_state() {
    let redacted = json!({
        "content": {},
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "sender": "@carl:example.com",
        "state_key": "hello there",
        "unsigned": unsigned(),
        "type": "m.made.up",
    });

    assert_matches!(
        from_json_value::<AnySyncTimelineEvent>(redacted),
        Ok(AnySyncTimelineEvent::State(state_ev))
    );
    assert_eq!(state_ev.event_id(), "$h29iv0s8:example.com");
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
        redacts: owned_event_id!("$143273582443PhrSn:example.com"),
        event_id: owned_event_id!("$h29iv0s8:example.com"),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
        sender: owned_user_id!("@carl:example.com"),
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

    assert_matches!(content.redact(&RedactionRules::V6), RedactedRoomMessageEventContent { .. });
}

#[test]
#[allow(deprecated)]
fn redact_state_content() {
    let json = json!({
        "creator": "@carl:example.com",
        "m.federate": true,
        "room_version": "4",
    });

    let raw_json = to_raw_json_value(&json).unwrap();
    let content = RoomCreateEventContent::from_parts("m.room.create", &raw_json).unwrap();

    assert_matches!(content.redact(&RedactionRules::V6), RoomCreateEventContent { creator, .. });
    assert_eq!(creator.unwrap(), "@carl:example.com");
}
