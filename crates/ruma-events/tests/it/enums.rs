use assert_matches::assert_matches;
use js_int::int;
use ruma_common::serde::{CanBeEmpty, test::serde_json_eq};
use ruma_events::{
    AnyMessageLikeEvent, AnyPossiblyRedactedStateEventContent, AnyStateEvent,
    AnySyncEphemeralRoomEvent, AnySyncMessageLikeEvent, AnySyncStateEvent, AnySyncTimelineEvent,
    AnyTimelineEvent, EmptyStateKey, EphemeralRoomEventType, GlobalAccountDataEventType,
    MessageLikeEvent, MessageLikeEventType, OriginalMessageLikeEvent, OriginalStateEvent,
    OriginalSyncMessageLikeEvent, OriginalSyncStateEvent, RoomAccountDataEventType, StateEvent,
    StateEventType, SyncMessageLikeEvent, SyncStateEvent, ToDeviceEventType,
    room::{
        member::{MembershipState, RoomMemberEventContent},
        message::{MessageType, RoomMessageEventContent},
        name::RoomNameEventContent,
        power_levels::RoomPowerLevelsEventContent,
        redaction::SyncRoomRedactionEvent,
    },
};
use serde_json::{Value as JsonValue, from_value as from_json_value, json};
use strass::assert_let;

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

    assert_let!(
        Ok(AnySyncTimelineEvent::State(AnySyncStateEvent::RoomPowerLevels(
            SyncStateEvent::Original(OriginalSyncStateEvent {
                content: RoomPowerLevelsEventContent { ban, .. },
                ..
            })
        ))) = from_json_value::<AnySyncTimelineEvent>(json_data)
    );
    assert_eq!(ban, int!(50));
}

#[test]
fn message_event_sync_deserialization() {
    let json_data = message_event_sync();

    assert_let!(
        Ok(AnySyncTimelineEvent::MessageLike(event)) =
            from_json_value::<AnySyncTimelineEvent>(json_data)
    );
    assert!(!event.is_redacted());

    assert_let!(
        AnySyncMessageLikeEvent::RoomMessage(SyncMessageLikeEvent::Original(
            OriginalSyncMessageLikeEvent {
                content: RoomMessageEventContent { msgtype: MessageType::Text(text_content), .. },
                ..
            },
        )) = event
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

    assert_let!(
        Ok(AnyTimelineEvent::MessageLike(event)) = from_json_value::<AnyTimelineEvent>(json_data)
    );
    assert!(!event.is_redacted());

    assert_let!(
        AnyMessageLikeEvent::RoomMessage(MessageLikeEvent::Original(OriginalMessageLikeEvent {
            content: RoomMessageEventContent { msgtype: MessageType::Text(text_content), .. },
            ..
        })) = event
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

    assert_let!(
        Ok(AnyTimelineEvent::State(state_ev)) = from_json_value::<AnyTimelineEvent>(redacted)
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

    assert_let!(
        Ok(AnySyncEphemeralRoomEvent::Typing(typing)) =
            from_json_value::<AnySyncEphemeralRoomEvent>(json_data)
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

fn timeline_event_with_unsigned(
    event_type: &str,
    content: JsonValue,
    unsigned: JsonValue,
) -> JsonValue {
    json!({
        "content": content,
        "event_id": "$152037280074GZeOm:localhost",
        "origin_server_ts": 1,
        "sender": "@example:localhost",
        "type": event_type,
        "unsigned": unsigned,
    })
}

fn redacted_because() -> JsonValue {
    json!({
        "type": "m.room.redaction",
        "content": { "reason": "redacted because" },
        "redacts": "$152037280074GZeOm:localhost",
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "sender": "@carl:example.com",
    })
}

#[test]
fn unsigned_membership_any_original_message_like() {
    let json = timeline_event_with_unsigned(
        "m.room.message",
        json!({ "body": "baba", "msgtype": "m.text" }),
        json!({ "membership": "join" }),
    );

    let event = from_json_value::<AnySyncTimelineEvent>(json).unwrap();
    assert_matches!(event, AnySyncTimelineEvent::MessageLike(_));
    assert_eq!(event.unsigned_membership(), Some(&MembershipState::Join));
}

#[test]
fn unsigned_membership_any_redacted_message_like() {
    let json = timeline_event_with_unsigned(
        "m.room.message",
        json!({}),
        json!({ "membership": "leave", "redacted_because": redacted_because() }),
    );

    let event = from_json_value::<AnySyncMessageLikeEvent>(json).unwrap();
    assert_matches!(event, AnySyncMessageLikeEvent::RoomMessage(SyncMessageLikeEvent::Redacted(_)));
    assert_eq!(event.unsigned_membership(), Some(&MembershipState::Leave));
    assert_eq!(event.transaction_id(), None);
}

#[test]
fn unsigned_membership_any_room_redaction() {
    let json = timeline_event_with_unsigned(
        "m.room.redaction",
        json!({ "redacts": "$other:localhost" }),
        json!({ "membership": "join" }),
    );

    let event = from_json_value::<AnySyncMessageLikeEvent>(json).unwrap();
    assert_matches!(event, AnySyncMessageLikeEvent::RoomRedaction(_));
    assert_eq!(event.unsigned_membership(), Some(&MembershipState::Join));
}

#[test]
fn unsigned_membership_any_custom_event_and_value() {
    let json = timeline_event_with_unsigned(
        "dev.ruma.custom",
        json!({}),
        json!({ "membership": "dev.ruma.custom_membership" }),
    );

    let event = from_json_value::<AnySyncMessageLikeEvent>(json).unwrap();
    assert_matches!(event, AnySyncMessageLikeEvent::_Custom(_));
    assert_eq!(event.unsigned_membership().map(|m| m.as_str()), Some("dev.ruma.custom_membership"));
}

#[test]
fn unsigned_membership_any_state() {
    let mut json = timeline_event_with_unsigned(
        "m.room.member",
        json!({ "membership": "join" }),
        json!({ "membership": "invite" }),
    );
    json["state_key"] = json!("@other:localhost");
    json["room_id"] = json!("!room:localhost");

    let event = from_json_value::<AnyTimelineEvent>(json).unwrap();
    assert_matches!(event, AnyTimelineEvent::State(AnyStateEvent::RoomMember(_)));
    assert_eq!(event.unsigned_membership(), Some(&MembershipState::Invite));
}

#[test]
fn unsigned_membership_any_redacted_state() {
    let mut json = timeline_event_with_unsigned(
        "m.room.name",
        json!({}),
        json!({ "membership": "ban", "redacted_because": redacted_because() }),
    );
    json["state_key"] = json!("");

    let event = from_json_value::<AnySyncTimelineEvent>(json).unwrap();
    assert_matches!(
        event,
        AnySyncTimelineEvent::State(AnySyncStateEvent::RoomName(SyncStateEvent::Redacted(_)))
    );
    assert_eq!(event.unsigned_membership(), Some(&MembershipState::Ban));
}

#[test]
fn unsigned_membership_any_redacted_room_redaction() {
    let json = timeline_event_with_unsigned(
        "m.room.redaction",
        json!({}),
        json!({ "membership": "knock", "redacted_because": redacted_because() }),
    );

    let event = from_json_value::<AnySyncMessageLikeEvent>(json).unwrap();
    assert_matches!(
        event,
        AnySyncMessageLikeEvent::RoomRedaction(SyncRoomRedactionEvent::Redacted(_))
    );
    assert_eq!(event.unsigned_membership(), Some(&MembershipState::Knock));
}

#[test]
fn unsigned_membership_typed_message_like() {
    let original = timeline_event_with_unsigned(
        "m.room.message",
        json!({ "body": "baba", "msgtype": "m.text" }),
        json!({ "membership": "join" }),
    );
    let event = from_json_value::<SyncMessageLikeEvent<RoomMessageEventContent>>(original).unwrap();
    assert_eq!(event.unsigned_membership(), Some(&MembershipState::Join));
    assert_let!(SyncMessageLikeEvent::Original(ev) = event);
    assert!(!ev.unsigned.is_empty());

    let mut redacted = timeline_event_with_unsigned(
        "m.room.message",
        json!({}),
        json!({ "membership": "leave", "redacted_because": redacted_because() }),
    );
    redacted["room_id"] = json!("!room:localhost");
    let event = from_json_value::<MessageLikeEvent<RoomMessageEventContent>>(redacted).unwrap();
    assert_matches!(event, MessageLikeEvent::Redacted(_));
    assert_eq!(event.unsigned_membership(), Some(&MembershipState::Leave));
}

#[test]
fn unsigned_membership_typed_state() {
    let mut original = timeline_event_with_unsigned(
        "m.room.member",
        json!({ "membership": "join" }),
        json!({ "membership": "invite" }),
    );
    original["state_key"] = json!("@other:localhost");
    let event = from_json_value::<SyncStateEvent<RoomMemberEventContent>>(original).unwrap();
    // `membership()` is the membership in the content, for the user in the `state_key`.
    assert_eq!(*event.membership(), MembershipState::Join);
    assert_let!(SyncStateEvent::Original(ev) = event);
    assert_eq!(ev.unsigned.membership, Some(MembershipState::Invite));
    assert!(!ev.unsigned.is_empty());

    let mut original = timeline_event_with_unsigned(
        "m.room.name",
        json!({ "name": "Somewhere" }),
        json!({ "membership": "join" }),
    );
    original["state_key"] = json!("");
    let event = from_json_value::<SyncStateEvent<RoomNameEventContent>>(original).unwrap();
    assert_let!(SyncStateEvent::Original(ev) = event);
    assert_eq!(ev.unsigned.membership, Some(MembershipState::Join));
    assert!(!ev.unsigned.is_empty());

    let mut redacted = timeline_event_with_unsigned(
        "m.room.name",
        json!({}),
        json!({ "membership": "ban", "redacted_because": redacted_because() }),
    );
    redacted["state_key"] = json!("");
    redacted["room_id"] = json!("!room:localhost");
    let event = from_json_value::<StateEvent<RoomNameEventContent>>(redacted).unwrap();
    assert_let!(StateEvent::Redacted(ev) = event);
    assert_eq!(ev.unsigned.membership, Some(MembershipState::Ban));
}

#[test]
fn unsigned_membership_typed_room_redaction() {
    let original = timeline_event_with_unsigned(
        "m.room.redaction",
        json!({ "redacts": "$other:localhost" }),
        json!({ "membership": "join" }),
    );
    let event = from_json_value::<SyncRoomRedactionEvent>(original).unwrap();
    assert_eq!(event.unsigned_membership(), Some(&MembershipState::Join));
    assert_let!(SyncRoomRedactionEvent::Original(ev) = event);
    assert!(!ev.unsigned.is_empty());

    let redacted = timeline_event_with_unsigned(
        "m.room.redaction",
        json!({}),
        json!({ "membership": "leave", "redacted_because": redacted_because() }),
    );
    let event = from_json_value::<SyncRoomRedactionEvent>(redacted).unwrap();
    assert_matches!(event, SyncRoomRedactionEvent::Redacted(_));
    assert_eq!(event.unsigned_membership(), Some(&MembershipState::Leave));
}

#[test]
fn unsigned_membership_any_missing() {
    let json = timeline_event_with_unsigned(
        "m.room.message",
        json!({ "body": "baba", "msgtype": "m.text" }),
        json!({}),
    );

    let event = from_json_value::<AnySyncTimelineEvent>(json).unwrap();
    assert_eq!(event.unsigned_membership(), None);
}
