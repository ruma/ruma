use assert_matches2::{assert_let, assert_matches};
use js_int::int;
use ruma_common::serde::test::serde_json_eq;
use ruma_events::{
    AnyMessageLikeEvent, AnyPossiblyRedactedStateEventContent, AnyStateEvent,
    AnySyncEphemeralRoomEvent, AnySyncMessageLikeEvent, AnySyncStateEvent, AnySyncTimelineEvent,
    AnyTimelineEvent, EmptyStateKey, EphemeralRoomEventType, GlobalAccountDataEventType,
    MessageLikeEvent, MessageLikeEventType, OriginalMessageLikeEvent, OriginalStateEvent,
    OriginalSyncMessageLikeEvent, OriginalSyncStateEvent, RoomAccountDataEventType, StateEvent,
    StateEventType, SyncMessageLikeEvent, SyncStateEvent, ToDeviceEventType,
    room::{
        message::{MessageType, RoomMessageEventContent},
        name::RoomNameEventContent,
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
        Ok(AnySyncTimelineEvent::State(AnySyncStateEvent::RoomPowerLevels(
            SyncStateEvent::Original(OriginalSyncStateEvent {
                content: RoomPowerLevelsEventContent { ban, .. },
                ..
            },)
        ),))
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
fn room_name_event_sync_deserialization() {
    let json = json!({
        "content": {
            "name": "Somewhere"
        },
        "event_id": "$152037280074GZeOm:localhost",
        "origin_server_ts": 1,
        "sender": "@example:localhost",
        "state_key": "",
        "type": "m.room.name",
        "unsigned": {
            "age": 1
        }
    });

    // Deserialize as timeline enum.
    assert_let!(Ok(AnySyncTimelineEvent::State(state_event)) = from_json_value(json.clone()));
    assert!(!state_event.is_redacted());
    assert_eq!(state_event.state_key(), "");
    assert_eq!(state_event.event_id(), "$152037280074GZeOm:localhost");
    assert_eq!(state_event.sender(), "@example:localhost");
    assert_eq!(state_event.event_type(), StateEventType::RoomName);
    assert_let!(AnySyncStateEvent::RoomName(SyncStateEvent::Original(event)) = &state_event);
    assert_eq!(event.content.name, "Somewhere");
    assert_let!(AnyPossiblyRedactedStateEventContent::RoomName(content) = state_event.content());
    assert_eq!(content.name.as_deref(), Some("Somewhere"));

    // Deserialize as state enum.
    assert_let!(Ok(AnySyncStateEvent::RoomName(state_event)) = from_json_value(json));
    assert_matches!(state_event.state_key(), EmptyStateKey);
    assert_eq!(state_event.event_id(), "$152037280074GZeOm:localhost");
    assert_eq!(state_event.sender(), "@example:localhost");
    assert_eq!(state_event.event_type(), StateEventType::RoomName);
    assert_let!(Some(event) = state_event.as_original());
    assert_eq!(event.content.name, "Somewhere");
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
fn room_name_event_deserialization() {
    let json = json!({
        "content": {
            "name": "Somewhere"
        },
        "event_id": "$152037280074GZeOm:localhost",
        "origin_server_ts": 1,
        "sender": "@example:localhost",
        "state_key": "",
        "room_id": "!room:room.com",
        "type": "m.room.name",
        "unsigned": {
            "age": 1
        }
    });

    // Deserialize as timeline enum.
    assert_let!(Ok(AnyTimelineEvent::State(state_event)) = from_json_value(json.clone()));
    assert!(!state_event.is_redacted());
    assert_eq!(state_event.state_key(), "");
    assert_eq!(state_event.room_id(), "!room:room.com");
    assert_eq!(state_event.event_id(), "$152037280074GZeOm:localhost");
    assert_eq!(state_event.sender(), "@example:localhost");
    assert_eq!(state_event.event_type(), StateEventType::RoomName);
    assert_let!(
        AnyStateEvent::RoomName(StateEvent::Original(OriginalStateEvent {
            content: RoomNameEventContent { name, .. },
            ..
        })) = &state_event
    );
    assert_eq!(name, "Somewhere");
    assert_let!(AnyPossiblyRedactedStateEventContent::RoomName(content) = state_event.content());
    assert_eq!(content.name.as_deref(), Some("Somewhere"));

    // Deserialize as state enum.
    assert_let!(Ok(AnyStateEvent::RoomName(state_event)) = from_json_value(json));
    assert_matches!(state_event.state_key(), EmptyStateKey);
    assert_eq!(state_event.room_id(), "!room:room.com");
    assert_eq!(state_event.event_id(), "$152037280074GZeOm:localhost");
    assert_eq!(state_event.sender(), "@example:localhost");
    assert_eq!(state_event.event_type(), StateEventType::RoomName);
    assert_let!(Some(event) = state_event.as_original());
    assert_eq!(event.content.name, "Somewhere");
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
    assert!(!state_ev.is_redacted());
    assert_eq!(state_ev.event_id(), "$h29iv0s8:example.com");
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
