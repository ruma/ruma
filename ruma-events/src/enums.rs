use ruma_events_macros::{event_content_enum, AnyEventDeserialize};
use serde::Serialize;

use crate::{
    event_kinds::{
        BasicEvent, EphemeralRoomEvent, MessageEvent, MessageEventStub, StateEvent, StateEventStub,
        StrippedStateEventStub, ToDeviceEvent,
    },
    presence::PresenceEvent,
    room::redaction::{RedactionEvent, RedactionEventStub},
};

event_content_enum! {
    /// Any basic event.
    name: AnyBasicEventContent,
    events: [
        "m.direct",
        "m.dummy",
        "m.ignored_user_list",
        "m.push_rules",
        "m.room_key",
        "m.tag",
    ]
}

event_content_enum! {
    /// Any ephemeral room event.
    name: AnyEphemeralRoomEventContent,
    events: [
        "m.fully_read",
        "m.receipt",
        "m.typing",
    ]
}

event_content_enum! {
    /// Any message event.
    name: AnyMessageEventContent,
    events: [
        "m.call.answer",
        "m.call.invite",
        "m.call.hangup",
        "m.call.candidates",
        "m.room.encrypted",
        "m.room.message",
        "m.room.message.feedback",
        "m.sticker",
    ]
}

event_content_enum! {
    /// Any state event.
    name: AnyStateEventContent,
    events: [
        "m.room.aliases",
        "m.room.avatar",
        "m.room.canonical_alias",
        "m.room.create",
        "m.room.encryption",
        "m.room.guest_access",
        "m.room.history_visibility",
        "m.room.join_rules",
        "m.room.member",
        "m.room.name",
        "m.room.pinned_events",
        "m.room.power_levels",
        "m.room.redaction",
        "m.room.server_acl",
        "m.room.third_party_invite",
        "m.room.tombstone",
        "m.room.topic",
    ]
}

event_content_enum! {
    /// Any to-device event.
    name: AnyToDeviceEventContent,
    events: [
        "m.dummy",
        "m.room_key",
        "m.room_key_request",
        "m.forwarded_room_key",
        "m.key.verification.request",
        "m.key.verification.start",
        "m.key.verification.cancel",
        "m.key.verification.accept",
        "m.key.verification.key",
        "m.key.verification.mac",
        "m.room.encrypted",
    ]
}

/// Any basic event, one that has no (well-known) fields outside of `content`.
pub type AnyBasicEvent = BasicEvent<AnyBasicEventContent>;

/// Any ephemeral room event.
pub type AnyEphemeralRoomEvent = EphemeralRoomEvent<AnyEphemeralRoomEventContent>;

/// Any message event.
pub type AnyMessageEvent = MessageEvent<AnyMessageEventContent>;

/// Any message event stub (message event without a `room_id`, as returned in `/sync` responses)
pub type AnyMessageEventStub = MessageEventStub<AnyMessageEventContent>;

/// Any state event.
pub type AnyStateEvent = StateEvent<AnyStateEventContent>;

/// Any state event stub (state event without a `room_id`, as returned in `/sync` responses)
pub type AnyStateEventStub = StateEventStub<AnyStateEventContent>;

/// Any stripped state event stub (stripped-down state event, as returned for rooms the user has
/// been invited to in `/sync` responses)
pub type AnyStrippedStateEventStub = StrippedStateEventStub<AnyStateEventContent>;

/// Any to-device event.
pub type AnyToDeviceEvent = ToDeviceEvent<AnyToDeviceEventContent>;

/// Any event.
#[derive(Clone, Debug, Serialize, AnyEventDeserialize)]
#[serde(untagged)]
pub enum AnyEvent {
    /// Any basic event.
    Basic(BasicEvent<AnyBasicEventContent>),
    /// `"m.presence"`, the only non-room event with a `sender` field.
    Presence(PresenceEvent),
    /// Any ephemeral room event.
    Ephemeral(EphemeralRoomEvent<AnyEphemeralRoomEventContent>),
    /// Any message event.
    Message(MessageEvent<AnyMessageEventContent>),
    /// `"m.room.redaction"`, the only room event with a `redacts` field.
    Redaction(RedactionEvent),
    /// Any state event.
    State(StateEvent<AnyStateEventContent>),
}

/// Any room event.
#[derive(Clone, Debug, Serialize, AnyEventDeserialize)]
#[serde(untagged)]
pub enum AnyRoomEvent {
    /// Any message event.
    Message(AnyMessageEvent),
    /// `"m.room.redaction"`, the only room event with a `redacts` field.
    Redaction(RedactionEvent),
    /// Any state event.
    State(AnyStateEvent),
}

/// Any room event stub (room event without a `room_id`, as returned in `/sync` responses)
#[derive(Clone, Debug, Serialize, AnyEventDeserialize)]
#[serde(untagged)]
pub enum AnyRoomEventStub {
    /// Any message event stub
    Message(AnyMessageEventStub),
    /// `"m.room.redaction"` stub
    Redaction(RedactionEventStub),
    /// Any state event stub
    StateEvent(AnyStateEventStub),
}

#[cfg(test)]
mod test {
    use std::convert::TryFrom;

    use matches::assert_matches;
    use ruma_identifiers::RoomAliasId;
    use serde_json::{from_value as from_json_value, json};

    use crate::{
        room::{
            aliases::AliasesEventContent,
            message::{MessageEventContent, TextMessageEventContent},
            power_levels::PowerLevelsEventContent,
        },
        AnyEvent, AnyMessageEventContent, AnyRoomEvent, AnyRoomEventStub, AnyStateEventContent,
        EventJson, MessageEvent, MessageEventStub, StateEvent, StateEventStub,
    };

    #[test]
    fn power_event_stub_deserialization() {
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
            from_json_value::<EventJson<AnyRoomEventStub>>(json_data)
                .unwrap()
                .deserialize()
                .unwrap(),
            AnyRoomEventStub::StateEvent(
                StateEventStub {
                    content: AnyStateEventContent::RoomPowerLevels(PowerLevelsEventContent {
                        ban, ..
                    }),
                    ..
                }
            )
            if ban == js_int::Int::new(50).unwrap()
        );
    }

    #[test]
    fn message_event_stub_deserialization() {
        let json_data = json!({
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
        });

        assert_matches!(
            from_json_value::<EventJson<AnyRoomEventStub>>(json_data)
                .unwrap()
                .deserialize()
                .unwrap(),
            AnyRoomEventStub::Message(
                MessageEventStub {
                    content: AnyMessageEventContent::RoomMessage(MessageEventContent::Text(TextMessageEventContent {
                        body,
                        formatted: Some(formatted),
                        relates_to: None,
                    })),
                    ..
                }
            )
            if body == "baba" && formatted.body == "<strong>baba</strong>"
        );
    }

    #[test]
    fn aliases_event_stub_deserialization() {
        let json_data = json!({
            "content": {
                "aliases": [ RoomAliasId::try_from("#somewhere:localhost").unwrap() ]
            },
            "event_id": "$152037280074GZeOm:localhost",
            "origin_server_ts": 1,
            "sender": "@example:localhost",
            "state_key": "",
            "type": "m.room.aliases",
            "unsigned": {
                "age": 1
            }
        });

        assert_matches!(
            from_json_value::<EventJson<AnyRoomEventStub>>(json_data)
                .unwrap()
                .deserialize()
                .unwrap(),
            AnyRoomEventStub::StateEvent(
                StateEventStub {
                    content: AnyStateEventContent::RoomAliases(AliasesEventContent {
                        aliases,
                    }),
                    ..
                }
            )
            if aliases == vec![ RoomAliasId::try_from("#somewhere:localhost").unwrap() ]
        );
    }

    #[test]
    fn message_room_event_deserialization() {
        let json_data = json!({
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
        });

        assert_matches!(
            from_json_value::<EventJson<AnyRoomEvent>>(json_data)
                .unwrap()
                .deserialize()
                .unwrap(),
            AnyRoomEvent::Message(
                MessageEvent {
                    content: AnyMessageEventContent::RoomMessage(MessageEventContent::Text(TextMessageEventContent {
                        body,
                        formatted: Some(formatted),
                        relates_to: None,
                    })),
                    ..
                }
            )
            if body == "baba" && formatted.body == "<strong>baba</strong>"
        );
    }

    #[test]
    fn alias_room_event_deserialization() {
        let json_data = json!({
            "content": {
                "aliases": [ "#somewhere:localhost" ]
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
        });

        assert_matches!(
            from_json_value::<EventJson<AnyRoomEvent>>(json_data)
                .unwrap()
                .deserialize()
                .unwrap(),
            AnyRoomEvent::State(
                StateEvent {
                    content: AnyStateEventContent::RoomAliases(AliasesEventContent {
                        aliases,
                    }),
                    ..
                }
            )
            if aliases == vec![ RoomAliasId::try_from("#somewhere:localhost").unwrap() ]
        );
    }

    #[test]
    fn message_event_deserialization() {
        let json_data = json!({
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
        });

        assert_matches!(
            from_json_value::<EventJson<AnyEvent>>(json_data)
                .unwrap()
                .deserialize()
                .unwrap(),
            AnyEvent::Message(
                MessageEvent {
                    content: AnyMessageEventContent::RoomMessage(MessageEventContent::Text(TextMessageEventContent {
                        body,
                        formatted: Some(formatted),
                        relates_to: None,
                    })),
                    ..
                }
            )
            if body == "baba" && formatted.body == "<strong>baba</strong>"
        );
    }

    #[test]
    fn alias_event_deserialization() {
        let json_data = json!({
            "content": {
                "aliases": [ "#somewhere:localhost" ]
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
        });

        assert_matches!(
            from_json_value::<EventJson<AnyEvent>>(json_data)
                .unwrap()
                .deserialize()
                .unwrap(),
            AnyEvent::State(
                StateEvent {
                    content: AnyStateEventContent::RoomAliases(AliasesEventContent {
                        aliases,
                    }),
                    ..
                }
            )
            if aliases == vec![ RoomAliasId::try_from("#somewhere:localhost").unwrap() ]
        );
    }
}
