use ruma_events_macros::event_enum;
use serde::{de, Serialize};
use serde_json::value::RawValue as RawJsonValue;

use crate::{from_raw_json_value, EventDeHelper};

event_enum! {
    /// Any basic event.
    name: AnyBasicEvent,
    events: [
        "m.direct",
        "m.dummy",
        "m.ignored_user_list",
        "m.presence",
        "m.push_rules",
        "m.room_key",
        "m.tag",
    ]
}

event_enum! {
    /// Any ephemeral room event.
    name: AnyEphemeralRoomEvent,
    events: [
        "m.fully_read",
        "m.receipt",
        "m.typing",
    ]
}

event_enum! {
    /// Any message event.
    name: AnyMessageEvent,
    events: [
        "m.call.answer",
        "m.call.invite",
        "m.call.hangup",
        "m.call.candidates",
        "m.room.encrypted",
        "m.room.message",
        "m.room.message.feedback",
        "m.room.redaction",
        "m.sticker",
    ]
}

event_enum! {
    /// Any state event.
    name: AnyStateEvent,
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
        "m.room.server_acl",
        "m.room.third_party_invite",
        "m.room.tombstone",
        "m.room.topic",
    ]
}

event_enum! {
    /// Any to-device event.
    name: AnyToDeviceEvent,
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

/// Any event.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum AnyEvent {
    /// Any basic event.
    Basic(AnyBasicEvent),
    /// Any ephemeral room event.
    Ephemeral(AnyEphemeralRoomEvent),
    /// Any message event.
    Message(AnyMessageEvent),
    /// Any state event.
    State(AnyStateEvent),
    /// Any message event that has been redacted.
    RedactedMessage(AnyRedactedMessageEvent),
    /// Any state event that has been redacted.
    RedactedState(AnyRedactedStateEvent),
}

/// Any room event.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum AnyRoomEvent {
    /// Any message event.
    Message(AnyMessageEvent),
    /// Any state event.
    State(AnyStateEvent),
    /// Any message event that has been redacted.
    RedactedMessage(AnyRedactedMessageEvent),
    /// Any state event that has been redacted.
    RedactedState(AnyRedactedStateEvent),
}

/// Any room event stub (room event without a `room_id`, as returned in `/sync` responses)
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum AnyRoomEventStub {
    /// Any message event stub
    Message(AnyMessageEventStub),
    /// Any state event stub
    State(AnyStateEventStub),
    /// Any message event stub that has been redacted.
    RedactedMessage(AnyRedactedMessageEventStub),
    /// Any state event stub that has been redacted.
    RedactedState(AnyRedactedStateEventStub),
}

// FIXME `#[serde(untagged)]` deserialization fails for these enums which
// is odd as we are doing basically the same thing here, investigate?
impl<'de> de::Deserialize<'de> for AnyEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let EventDeHelper { state_key, event_id, room_id, unsigned, .. } =
            from_raw_json_value(&json)?;

        // Determine whether the event is a state, message, ephemeral, or basic event
        // based on the fields present.
        if state_key.is_some() {
            if let Some(unsigned) = unsigned {
                if unsigned.redacted_because.is_some() {
                    return Ok(AnyEvent::RedactedState(from_raw_json_value(&json)?));
                }
            }
            Ok(AnyEvent::State(from_raw_json_value(&json)?))
        } else if event_id.is_some() {
            if let Some(unsigned) = unsigned {
                if unsigned.redacted_because.is_some() {
                    return Ok(AnyEvent::RedactedMessage(from_raw_json_value(&json)?));
                }
            }
            Ok(AnyEvent::Message(from_raw_json_value(&json)?))
        } else if room_id.is_some() {
            Ok(AnyEvent::Ephemeral(from_raw_json_value(&json)?))
        } else {
            Ok(AnyEvent::Basic(from_raw_json_value(&json)?))
        }
    }
}

impl<'de> de::Deserialize<'de> for AnyRoomEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let EventDeHelper { state_key, unsigned, .. } = from_raw_json_value(&json)?;

        if state_key.is_some() {
            if let Some(unsigned) = unsigned {
                if unsigned.redacted_because.is_some() {
                    return Ok(AnyRoomEvent::RedactedState(from_raw_json_value(&json)?));
                }
            }
            Ok(AnyRoomEvent::State(from_raw_json_value(&json)?))
        } else {
            if let Some(unsigned) = unsigned {
                if unsigned.redacted_because.is_some() {
                    return Ok(AnyRoomEvent::RedactedMessage(from_raw_json_value(&json)?));
                }
            }
            Ok(AnyRoomEvent::Message(from_raw_json_value(&json)?))
        }
    }
}

impl<'de> de::Deserialize<'de> for AnyRoomEventStub {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let EventDeHelper { state_key, unsigned, .. } = from_raw_json_value(&json)?;

        if state_key.is_some() {
            if let Some(unsigned) = unsigned {
                if unsigned.redacted_because.is_some() {
                    return Ok(AnyRoomEventStub::RedactedState(from_raw_json_value(&json)?));
                }
            }
            Ok(AnyRoomEventStub::State(from_raw_json_value(&json)?))
        } else {
            if let Some(unsigned) = unsigned {
                if unsigned.redacted_because.is_some() {
                    return Ok(AnyRoomEventStub::RedactedMessage(from_raw_json_value(&json)?));
                }
            }
            Ok(AnyRoomEventStub::Message(from_raw_json_value(&json)?))
        }
    }
}
