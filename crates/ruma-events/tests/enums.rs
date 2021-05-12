use js_int::uint;
use matches::assert_matches;
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_identifiers::{event_id, room_alias_id, room_id, user_id};
use serde_json::{from_value as from_json_value, json, Value as JsonValue};

use ruma_events::{
    room::{
        aliases::AliasesEventContent,
        message::{MessageEventContent, MessageType, TextMessageEventContent},
        power_levels::PowerLevelsEventContent,
    },
    AnyEphemeralRoomEvent, AnyMessageEvent, AnyRoomEvent, AnyStateEvent, AnyStateEventContent,
    AnySyncMessageEvent, AnySyncRoomEvent, AnySyncStateEvent, MessageEvent, StateEvent,
    SyncMessageEvent, SyncStateEvent, Unsigned,
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
        "state_key": "",
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
        "state_key": "",
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
        from_json_value::<AnySyncRoomEvent>(json_data),
        Ok(AnySyncRoomEvent::State(
            AnySyncStateEvent::RoomPowerLevels(SyncStateEvent {
                content: PowerLevelsEventContent {
                    ban, ..
                },
                ..
            })
        ))
        if ban == js_int::Int::new(50).unwrap()
    );
}

#[test]
fn message_event_sync_deserialization() {
    let json_data = message_event_sync();

    assert_matches!(
        from_json_value::<AnySyncRoomEvent>(json_data),
        Ok(AnySyncRoomEvent::Message(
            AnySyncMessageEvent::RoomMessage(SyncMessageEvent {
                content: MessageEventContent {
                    msgtype: MessageType::Text(TextMessageEventContent {
                        body,
                        formatted: Some(formatted),
                        ..
                    }),
                    ..
                },
                ..
            })
        ))
        if body == "baba" && formatted.body == "<strong>baba</strong>"
    );
}

#[test]
fn aliases_event_sync_deserialization() {
    let json_data = aliases_event_sync();

    assert_matches!(
        from_json_value::<AnySyncRoomEvent>(json_data),
        Ok(AnySyncRoomEvent::State(
            AnySyncStateEvent::RoomAliases(SyncStateEvent {
                content: AliasesEventContent {
                    aliases,
                    ..
                },
                ..
            })
        ))
        if aliases == vec![ room_alias_id!("#somewhere:localhost") ]
    );
}

#[test]
fn message_room_event_deserialization() {
    let json_data = message_event();

    assert_matches!(
        from_json_value::<AnyRoomEvent>(json_data),
        Ok(AnyRoomEvent::Message(
            AnyMessageEvent::RoomMessage(MessageEvent {
                content: MessageEventContent {
                    msgtype: MessageType::Text(TextMessageEventContent {
                        body,
                        formatted: Some(formatted),
                        ..
                    }),
                    ..
                },
                ..
            })
        ))
        if body == "baba" && formatted.body == "<strong>baba</strong>"
    );
}

#[test]
fn message_event_serialization() {
    let event = MessageEvent {
        content: MessageEventContent::text_plain("test"),
        event_id: event_id!("$1234:example.com"),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(0)),
        room_id: room_id!("!roomid:example.com"),
        sender: user_id!("@test:example.com"),
        unsigned: Unsigned::default(),
    };

    assert_eq!(
        serde_json::to_string(&event).expect("Failed to serialize message event"),
        r#"{"type":"m.room.message","content":{"msgtype":"m.text","body":"test"},"event_id":"$1234:example.com","sender":"@test:example.com","origin_server_ts":0,"room_id":"!roomid:example.com"}"#
    )
}

#[test]
fn alias_room_event_deserialization() {
    let json_data = aliases_event();

    assert_matches!(
        from_json_value::<AnyRoomEvent>(json_data),
        Ok(AnyRoomEvent::State(
            AnyStateEvent::RoomAliases(StateEvent {
                content: AliasesEventContent {
                    aliases,
                    ..
                },
                ..
            })
        ))
        if aliases == vec![ room_alias_id!("#somewhere:localhost") ]
    );
}

#[test]
fn message_event_deserialization() {
    let json_data = message_event();

    assert_matches!(
        from_json_value::<AnyRoomEvent>(json_data),
        Ok(AnyRoomEvent::Message(
            AnyMessageEvent::RoomMessage(MessageEvent {
                content: MessageEventContent {
                    msgtype: MessageType::Text(TextMessageEventContent {
                        body,
                        formatted: Some(formatted),
                        ..
                    }),
                    ..
                },
                ..
            })
        ))
        if body == "baba" && formatted.body == "<strong>baba</strong>"
    );
}

#[test]
fn alias_event_deserialization() {
    let json_data = aliases_event();

    assert_matches!(
        from_json_value::<AnyRoomEvent>(json_data),
        Ok(AnyRoomEvent::State(
            AnyStateEvent::RoomAliases(StateEvent {
                content: AliasesEventContent {
                    aliases,
                    ..
                },
                ..
            })
        ))
        if aliases == vec![ room_alias_id!("#somewhere:localhost") ]
    );
}

#[test]
fn alias_event_field_access() {
    let json_data = aliases_event();

    assert_matches!(
        from_json_value::<AnyRoomEvent>(json_data.clone()),
        Ok(AnyRoomEvent::State(state_event))
        if state_event.state_key() == ""
            && state_event.room_id() == &room_id!("!room:room.com")
            && state_event.event_id() == &event_id!("$152037280074GZeOm:localhost")
            && state_event.sender() == &user_id!("@example:localhost")
    );

    let deser = from_json_value::<AnyStateEvent>(json_data).unwrap();
    if let AnyStateEventContent::RoomAliases(AliasesEventContent { aliases, .. }) = deser.content()
    {
        assert_eq!(aliases, vec![room_alias_id!("#somewhere:localhost")])
    } else {
        panic!("the `Any*Event` enum's accessor methods may have been altered")
    }
    assert_eq!(deser.event_type(), "m.room.aliases");
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

    assert_matches!(
        from_json_value::<AnyEphemeralRoomEvent>(json_data),
        Ok(ephem @ AnyEphemeralRoomEvent::Typing(_))
        if ephem.room_id() == &room_id!("!jEsUZKDJdhlrceRyVU:example.org")
    );
}
