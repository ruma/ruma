use assert_matches::assert_matches;
use js_int::{int, uint};
use ruma_common::{
    event_id,
    events::{MessageLikeEvent, StateEvent, SyncMessageLikeEvent, SyncStateEvent},
    room_alias_id, room_id,
    serde::test::serde_json_eq,
    user_id,
};
use serde_json::{from_value as from_json_value, json, Value as JsonValue};

use ruma_common::{
    events::{
        room::{
            aliases::RoomAliasesEventContent,
            message::{MessageType, RoomMessageEventContent},
            power_levels::RoomPowerLevelsEventContent,
        },
        AnyEphemeralRoomEvent, AnyMessageLikeEvent, AnyRoomEvent, AnyStateEvent,
        AnySyncMessageLikeEvent, AnySyncRoomEvent, AnySyncStateEvent, EphemeralRoomEventType,
        GlobalAccountDataEventType, MessageLikeEventType, MessageLikeUnsigned,
        OriginalMessageLikeEvent, OriginalStateEvent, OriginalSyncMessageLikeEvent,
        OriginalSyncStateEvent, RoomAccountDataEventType, StateEventType, ToDeviceEventType,
    },
    MilliSecondsSinceUnixEpoch,
};

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

    let ban = assert_matches!(
        from_json_value::<AnySyncRoomEvent>(json_data),
        Ok(AnySyncRoomEvent::State(
            AnySyncStateEvent::RoomPowerLevels(SyncStateEvent::Original(
                OriginalSyncStateEvent {
                    content: RoomPowerLevelsEventContent {
                        ban, ..
                    },
                    ..
                },
            )),
        )) => ban
    );
    assert_eq!(ban, int!(50));
}

#[test]
fn message_event_sync_deserialization() {
    let json_data = message_event_sync();

    let text_content = assert_matches!(
        from_json_value::<AnySyncRoomEvent>(json_data),
        Ok(AnySyncRoomEvent::MessageLike(
            AnySyncMessageLikeEvent::RoomMessage(SyncMessageLikeEvent::Original(
                OriginalSyncMessageLikeEvent {
                    content: RoomMessageEventContent {
                        msgtype: MessageType::Text(text_content),
                        ..
                    },
                    ..
                },
            ))
        )) => text_content
    );
    assert_eq!(text_content.body, "baba");
    let formatted = text_content.formatted.unwrap();
    assert_eq!(formatted.body, "<strong>baba</strong>");
}

#[test]
fn aliases_event_sync_deserialization() {
    let json_data = aliases_event_sync();

    let ev = assert_matches!(
        from_json_value::<AnySyncRoomEvent>(json_data),
        Ok(AnySyncRoomEvent::State(AnySyncStateEvent::RoomAliases(SyncStateEvent::Original(
            ev,
        )))) => ev
    );

    assert_eq!(ev.content.aliases, vec![room_alias_id!("#somewhere:localhost")]);
}

#[test]
fn message_room_event_deserialization() {
    let json_data = message_event();

    let text_content = assert_matches!(
        from_json_value::<AnyRoomEvent>(json_data),
        Ok(AnyRoomEvent::MessageLike(
            AnyMessageLikeEvent::RoomMessage(MessageLikeEvent::Original(
                OriginalMessageLikeEvent {
                    content: RoomMessageEventContent {
                        msgtype: MessageType::Text(text_content),
                        ..
                    },
                    ..
                },
            ))
        )) => text_content
    );
    assert_eq!(text_content.body, "baba");
    let formatted = text_content.formatted.unwrap();
    assert_eq!(formatted.body, "<strong>baba</strong>");
}

#[test]
fn message_event_serialization() {
    let event = OriginalMessageLikeEvent {
        content: RoomMessageEventContent::text_plain("test"),
        event_id: event_id!("$1234:example.com").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(0)),
        room_id: room_id!("!roomid:example.com").to_owned(),
        sender: user_id!("@test:example.com").to_owned(),
        unsigned: MessageLikeUnsigned::default(),
    };

    #[cfg(not(feature = "unstable-msc1767"))]
    assert_eq!(
        serde_json::to_string(&event).expect("Failed to serialize message event"),
        r#"{"type":"m.room.message","content":{"msgtype":"m.text","body":"test"},"event_id":"$1234:example.com","sender":"@test:example.com","origin_server_ts":0,"room_id":"!roomid:example.com"}"#
    );

    #[cfg(feature = "unstable-msc1767")]
    assert_eq!(
        serde_json::to_string(&event).expect("Failed to serialize message event"),
        r#"{"type":"m.room.message","content":{"msgtype":"m.text","body":"test","org.matrix.msc1767.text":"test"},"event_id":"$1234:example.com","sender":"@test:example.com","origin_server_ts":0,"room_id":"!roomid:example.com"}"#
    );
}

#[test]
fn alias_room_event_deserialization() {
    let json_data = aliases_event();

    let aliases = assert_matches!(
        from_json_value::<AnyRoomEvent>(json_data),
        Ok(AnyRoomEvent::State(
            AnyStateEvent::RoomAliases(StateEvent::Original(OriginalStateEvent {
                content: RoomAliasesEventContent {
                    aliases,
                    ..
                },
                ..
            }))
        )) => aliases
    );
    assert_eq!(aliases, vec![room_alias_id!("#somewhere:localhost")]);
}

#[test]
fn message_event_deserialization() {
    let json_data = message_event();

    let text_content = assert_matches!(
        from_json_value::<AnyRoomEvent>(json_data),
        Ok(AnyRoomEvent::MessageLike(
            AnyMessageLikeEvent::RoomMessage(MessageLikeEvent::Original(OriginalMessageLikeEvent {
                content: RoomMessageEventContent {
                    msgtype: MessageType::Text(text_content),
                    ..
                },
                ..
            }))
        )) => text_content
    );
    assert_eq!(text_content.body, "baba");
    let formatted = text_content.formatted.unwrap();
    assert_eq!(formatted.body, "<strong>baba</strong>");
}

#[test]
fn alias_event_deserialization() {
    let json_data = aliases_event();

    let aliases = assert_matches!(
        from_json_value::<AnyRoomEvent>(json_data),
        Ok(AnyRoomEvent::State(
            AnyStateEvent::RoomAliases(StateEvent::Original(OriginalStateEvent {
                content: RoomAliasesEventContent {
                    aliases,
                    ..
                },
                ..
            }))
        )) => aliases
    );
    assert_eq!(aliases, vec![room_alias_id!("#somewhere:localhost")]);
}

#[test]
fn alias_event_field_access() {
    let json_data = aliases_event();

    let state_event = assert_matches!(
        from_json_value::<AnyRoomEvent>(json_data.clone()),
        Ok(AnyRoomEvent::State(state_event)) => state_event
    );
    assert_eq!(state_event.state_key(), "room.com");
    assert_eq!(state_event.room_id(), "!room:room.com");
    assert_eq!(state_event.event_id(), "$152037280074GZeOm:localhost");
    assert_eq!(state_event.sender(), "@example:localhost");

    let deser = from_json_value::<AnyStateEvent>(json_data).unwrap();
    let ev = assert_matches!(&deser, AnyStateEvent::RoomAliases(StateEvent::Original(ev)) => ev);
    assert_eq!(ev.content.aliases, vec![room_alias_id!("#somewhere:localhost")]);
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
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "type": "m.typing"
    });

    let ephem = assert_matches!(
        from_json_value::<AnyEphemeralRoomEvent>(json_data),
        Ok(ephem @ AnyEphemeralRoomEvent::Typing(_)) => ephem
    );
    assert_eq!(ephem.room_id(), "!jEsUZKDJdhlrceRyVU:example.org");
}

#[test]
#[allow(deprecated)]
fn serialize_and_deserialize_from_display_form() {
    use ruma_common::events::EventType;

    serde_json_eq(EventType::CallAnswer, json!("m.call.answer"));
    serde_json_eq(MessageLikeEventType::CallAnswer, json!("m.call.answer"));
    serde_json_eq(EventType::CallCandidates, json!("m.call.candidates"));
    serde_json_eq(EventType::CallHangup, json!("m.call.hangup"));
    serde_json_eq(EventType::CallInvite, json!("m.call.invite"));
    serde_json_eq(EventType::Direct, json!("m.direct"));
    serde_json_eq(GlobalAccountDataEventType::Direct, json!("m.direct"));
    serde_json_eq(EventType::Dummy, json!("m.dummy"));
    serde_json_eq(EventType::ForwardedRoomKey, json!("m.forwarded_room_key"));
    serde_json_eq(EventType::FullyRead, json!("m.fully_read"));
    serde_json_eq(RoomAccountDataEventType::FullyRead, json!("m.fully_read"));
    serde_json_eq(EventType::KeyVerificationAccept, json!("m.key.verification.accept"));
    serde_json_eq(EventType::KeyVerificationCancel, json!("m.key.verification.cancel"));
    serde_json_eq(EventType::KeyVerificationDone, json!("m.key.verification.done"));
    serde_json_eq(EventType::KeyVerificationKey, json!("m.key.verification.key"));
    serde_json_eq(ToDeviceEventType::KeyVerificationKey, json!("m.key.verification.key"));
    serde_json_eq(EventType::KeyVerificationMac, json!("m.key.verification.mac"));
    serde_json_eq(EventType::KeyVerificationReady, json!("m.key.verification.ready"));
    serde_json_eq(EventType::KeyVerificationRequest, json!("m.key.verification.request"));
    serde_json_eq(EventType::KeyVerificationStart, json!("m.key.verification.start"));
    serde_json_eq(EventType::IgnoredUserList, json!("m.ignored_user_list"));
    serde_json_eq(EventType::PolicyRuleRoom, json!("m.policy.rule.room"));
    serde_json_eq(EventType::PolicyRuleServer, json!("m.policy.rule.server"));
    serde_json_eq(EventType::PolicyRuleUser, json!("m.policy.rule.user"));
    serde_json_eq(EventType::Presence, json!("m.presence"));
    serde_json_eq(EventType::PushRules, json!("m.push_rules"));
    serde_json_eq(EventType::Receipt, json!("m.receipt"));
    serde_json_eq(EventType::RoomAliases, json!("m.room.aliases"));
    serde_json_eq(EventType::RoomAvatar, json!("m.room.avatar"));
    serde_json_eq(EventType::RoomCanonicalAlias, json!("m.room.canonical_alias"));
    serde_json_eq(EventType::RoomCreate, json!("m.room.create"));
    serde_json_eq(StateEventType::RoomCreate, json!("m.room.create"));
    serde_json_eq(EventType::RoomEncrypted, json!("m.room.encrypted"));
    serde_json_eq(EventType::RoomEncryption, json!("m.room.encryption"));
    serde_json_eq(EventType::RoomGuestAccess, json!("m.room.guest_access"));
    serde_json_eq(EventType::RoomHistoryVisibility, json!("m.room.history_visibility"));
    serde_json_eq(EventType::RoomJoinRules, json!("m.room.join_rules"));
    serde_json_eq(EventType::RoomMember, json!("m.room.member"));
    serde_json_eq(EventType::RoomMessage, json!("m.room.message"));
    serde_json_eq(EventType::RoomMessageFeedback, json!("m.room.message.feedback"));
    serde_json_eq(EventType::RoomName, json!("m.room.name"));
    serde_json_eq(EventType::RoomPinnedEvents, json!("m.room.pinned_events"));
    serde_json_eq(EventType::RoomPowerLevels, json!("m.room.power_levels"));
    serde_json_eq(EventType::RoomRedaction, json!("m.room.redaction"));
    serde_json_eq(EventType::RoomServerAcl, json!("m.room.server_acl"));
    serde_json_eq(EventType::RoomThirdPartyInvite, json!("m.room.third_party_invite"));
    serde_json_eq(EventType::RoomTombstone, json!("m.room.tombstone"));
    serde_json_eq(EventType::RoomTopic, json!("m.room.topic"));
    serde_json_eq(EventType::RoomKey, json!("m.room_key"));
    serde_json_eq(EventType::RoomKeyRequest, json!("m.room_key_request"));
    serde_json_eq(EventType::Sticker, json!("m.sticker"));
    serde_json_eq(EventType::Tag, json!("m.tag"));
    serde_json_eq(EventType::Typing, json!("m.typing"));
    serde_json_eq(EphemeralRoomEventType::Typing, json!("m.typing"));
}
