use assert_matches2::assert_matches;
use js_int::int;
use ruma_common::{room_alias_id, serde::test::serde_json_eq};
use ruma_events::{
    AnyMessageLikeEvent, AnyStateEvent, AnySyncEphemeralRoomEvent, AnySyncMessageLikeEvent,
    AnySyncStateEvent, AnySyncTimelineEvent, AnyTimelineEvent, EphemeralRoomEventType,
    GlobalAccountDataEventType, MessageLikeEvent, MessageLikeEventType, OriginalMessageLikeEvent,
    OriginalSyncMessageLikeEvent, RoomAccountDataEventType, StateEvent, StateEventType,
    SyncMessageLikeEvent, SyncStateEvent, ToDeviceEventType,
    room::{
        aliases::RoomAliasesEventContent,
        message::{MessageType, RoomMessageEventContent},
        power_levels::RoomPowerLevelsEventContent,
    },
};
use serde_json::{Value as JsonValue, from_value as from_json_value, json};

fn message_event() -> JsonValue {
    json!({
        "content": {
            "body": "baba",
            "format": "org.matrix.custom.html",
            "formatted_body": "<strong>baba</strong>",
            "msgtype": "m.text"
        },
        "event_id": "$152037280074GZeOm:localhost",
        "origin_server_ts": 1,
        "sender": "@example:localhost",
        "room_id": "!room:room.com",
        "type": "m.room.message",
        "unsigned": {
            "age": 1
        }
    })
}

fn message_event_sync() -> JsonValue {
    json!({
        "content": {
            "body": "baba",
            "format": "org.matrix.custom.html",
            "formatted_body": "<strong>baba</strong>",
            "msgtype": "m.text"
        },
        "event_id": "$152037280074GZeOm:localhost",
        "origin_server_ts": 1,
        "sender": "@example:localhost",
        "type": "m.room.message",
        "unsigned": {
            "age": 1
        }
    })
}

fn aliases_event() -> JsonValue {
    json!({
        "content": {
            "aliases": ["#somewhere:localhost"]
        },
        "event_id": "$152037280074GZeOm:localhost",
        "origin_server_ts": 1,
        "sender": "@example:localhost",
        "state_key": "room.com",
        "room_id": "!room:room.com",
        "type": "m.room.aliases",
        "unsigned": {
            "age": 1
        }
    })
}

fn aliases_event_sync() -> JsonValue {
    json!({
        "content": {
            "aliases": ["#somewhere:localhost"]
        },
        "event_id": "$152037280074GZeOm:localhost",
        "origin_server_ts": 1,
        "sender": "@example:localhost",
        "state_key": "example.com",
        "type": "m.room.aliases",
        "unsigned": {
            "age": 1
        }
    })
}

#[test]
fn power_event_sync_deserialization() {
    let json_data = json!({
        "content": {
            "ban": 50,
            "events": {
                "m.room.avatar": 50,
                "m.room.canonical_alias": 50,
                "m.room.history_visibility": 100,
                "m.room.name": 50,
                "m.room.power_levels": 100
            },
            "events_default": 0,
            "invite": 0,
            "kick": 50,
            "redact": 50,
            "state_default": 50,
            "users": {
                "@example:localhost": 100
            },
            "users_default": 0
        },
        "event_id": "$15139375512JaHAW:localhost",
        "origin_server_ts": 45,
        "sender": "@example:localhost",
        "state_key": "",
        "type": "m.room.power_levels",
        "unsigned": {
            "age": 45
        }
    });

    assert_matches!(
        from_json_value::<AnySyncTimelineEvent>(json_data),
        Ok(AnySyncTimelineEvent::State(AnySyncStateEvent::RoomPowerLevels(SyncStateEvent {
            content: RoomPowerLevelsEventContent { ban, .. },
            ..
        })))
    );
    assert_eq!(ban, int!(50));
}

#[test]
fn message_event_sync_deserialization() {
    let json_data = message_event_sync();

    assert_matches!(
        from_json_value::<AnySyncTimelineEvent>(json_data),
        Ok(AnySyncTimelineEvent::MessageLike(event))
    );
    assert!(!event.is_redacted());

    assert_matches!(
        event,
        AnySyncMessageLikeEvent::RoomMessage(SyncMessageLikeEvent::Original(
            OriginalSyncMessageLikeEvent {
                content: RoomMessageEventContent { msgtype: MessageType::Text(text_content), .. },
                ..
            },
        ))
    );
    assert_eq!(text_content.body, "baba");
    let formatted = text_content.formatted.unwrap();
    assert_eq!(formatted.body, "<strong>baba</strong>");
}

#[test]
fn aliases_event_sync_deserialization() {
    let json_data = aliases_event_sync();

    assert_matches!(
        from_json_value::<AnySyncTimelineEvent>(json_data),
        Ok(AnySyncTimelineEvent::State(state_event))
    );

    assert_matches!(state_event, AnySyncStateEvent::RoomAliases(ev));
    assert_eq!(ev.content.aliases.unwrap(), vec![room_alias_id!("#somewhere:localhost")]);
}

#[test]
fn message_event_deserialization() {
    let json_data = message_event();

    assert_matches!(
        from_json_value::<AnyTimelineEvent>(json_data),
        Ok(AnyTimelineEvent::MessageLike(event))
    );
    assert!(!event.is_redacted());

    assert_matches!(
        event,
        AnyMessageLikeEvent::RoomMessage(MessageLikeEvent::Original(OriginalMessageLikeEvent {
            content: RoomMessageEventContent { msgtype: MessageType::Text(text_content), .. },
            ..
        }))
    );
    assert_eq!(text_content.body, "baba");
    let formatted = text_content.formatted.unwrap();
    assert_eq!(formatted.body, "<strong>baba</strong>");
}

#[test]
fn message_event_serialization() {
    let content = RoomMessageEventContent::text_plain("test");

    assert_eq!(
        serde_json::to_string(&content).expect("Failed to serialize message event content"),
        r#"{"msgtype":"m.text","body":"test"}"#
    );
}

#[test]
fn alias_event_deserialization() {
    let json_data = aliases_event();

    assert_matches!(
        from_json_value::<AnyTimelineEvent>(json_data),
        Ok(AnyTimelineEvent::State(event))
    );

    assert_matches!(
        event,
        AnyStateEvent::RoomAliases(StateEvent {
            content: RoomAliasesEventContent { aliases, .. },
            ..
        })
    );
    assert_eq!(aliases.unwrap(), vec![room_alias_id!("#somewhere:localhost")]);
}

#[test]
fn custom_state_event_deserialization() {
    let redacted = json!({
        "content": {},
        "event_id": "$h29iv0s8:example.com",
        "room_id": "!room:room.com",
        "origin_server_ts": 1,
        "sender": "@carl:example.com",
        "state_key": "hello there",
        "type": "m.made.up",
    });

    assert_matches!(
        from_json_value::<AnyTimelineEvent>(redacted),
        Ok(AnyTimelineEvent::State(state_ev))
    );
    assert_eq!(state_ev.event_id(), "$h29iv0s8:example.com");
}

#[test]
fn alias_event_field_access() {
    let json_data = aliases_event();

    assert_matches!(
        from_json_value::<AnyTimelineEvent>(json_data.clone()),
        Ok(AnyTimelineEvent::State(state_event))
    );
    assert_eq!(state_event.state_key(), "room.com");
    assert_eq!(state_event.room_id(), "!room:room.com");
    assert_eq!(state_event.event_id(), "$152037280074GZeOm:localhost");
    assert_eq!(state_event.sender(), "@example:localhost");

    let deser = from_json_value::<AnyStateEvent>(json_data).unwrap();
    assert_matches!(&deser, AnyStateEvent::RoomAliases(ev));
    assert_eq!(ev.content.aliases.as_ref().unwrap(), &[room_alias_id!("#somewhere:localhost")]);
    assert_eq!(deser.event_type().to_string(), "m.room.aliases");
}

#[test]
fn ephemeral_event_deserialization() {
    let json_data = json!({
        "content": {
            "user_ids": [
                "@alice:matrix.org",
                "@bob:example.com"
            ]
        },
        "type": "m.typing"
    });

    assert_matches!(
        from_json_value::<AnySyncEphemeralRoomEvent>(json_data),
        Ok(AnySyncEphemeralRoomEvent::Typing(typing))
    );
    assert_eq!(typing.content.user_ids.len(), 2);
}

#[test]
fn serialize_and_deserialize_from_display_form() {
    serde_json_eq(MessageLikeEventType::CallAnswer, json!("m.call.answer"));
    serde_json_eq(GlobalAccountDataEventType::Direct, json!("m.direct"));
    serde_json_eq(RoomAccountDataEventType::FullyRead, json!("m.fully_read"));
    serde_json_eq(ToDeviceEventType::KeyVerificationKey, json!("m.key.verification.key"));
    serde_json_eq(StateEventType::RoomCreate, json!("m.room.create"));
    serde_json_eq(EphemeralRoomEventType::Typing, json!("m.typing"));
}
