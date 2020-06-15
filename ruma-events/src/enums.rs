use ruma_events_macros::event_content_enum;
use serde::{
    de::{self, Error as _},
    Serialize,
};
use serde_json::{from_value as from_json_value, Value as JsonValue};

use crate::{
    event_kinds::{
        BasicEvent, EphemeralRoomEvent, MessageEvent, MessageEventStub, StateEvent, StateEventStub,
        StrippedStateEventStub, ToDeviceEvent,
    },
    presence::PresenceEvent,
    room::redaction::{RedactionEvent, RedactionEventStub},
    util,
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
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum AnyEvent {
    /// Any basic event.
    Basic(AnyBasicEvent),
    /// `"m.presence"`, the only non-room event with a `sender` field.
    Presence(PresenceEvent),
    /// Any ephemeral room event.
    Ephemeral(AnyEphemeralRoomEvent),
    /// Any message event.
    Message(AnyMessageEvent),
    /// `"m.room.redaction"`, the only room event with a `redacts` field.
    Redaction(RedactionEvent),
    /// Any state event.
    State(AnyStateEvent),
}

/// Any room event.
#[derive(Clone, Debug, Serialize)]
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
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum AnyRoomEventStub {
    /// Any message event stub
    Message(AnyMessageEventStub),
    /// `"m.room.redaction"` stub
    Redaction(RedactionEventStub),
    /// Any state event stub
    State(AnyStateEventStub),
}

// FIXME `#[serde(untagged)]` deserialization fails for these enums which
// is odd as we are doing basically the same thing here, investigate?
impl<'de> de::Deserialize<'de> for AnyEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = JsonValue::deserialize(deserializer)?;
        let ev_type: String = util::get_field(&json, "type")?;

        match ev_type.as_str() {
            "m.room.redaction" => {
                Ok(AnyEvent::Redaction(from_json_value(json).map_err(D::Error::custom)?))
            }
            "m.presence" => {
                Ok(AnyEvent::Presence(from_json_value(json).map_err(D::Error::custom)?))
            }
            ev_type if AnyBasicEventContent::is_compatible(ev_type) => {
                Ok(AnyEvent::Basic(from_json_value(json).map_err(D::Error::custom)?))
            }
            ev_type if AnyEphemeralRoomEventContent::is_compatible(ev_type) => {
                Ok(AnyEvent::Ephemeral(from_json_value(json).map_err(D::Error::custom)?))
            }
            ev_type if AnyMessageEventContent::is_compatible(ev_type) => {
                Ok(AnyEvent::Message(from_json_value(json).map_err(D::Error::custom)?))
            }
            ev_type if AnyStateEventContent::is_compatible(ev_type) => {
                Ok(AnyEvent::State(from_json_value(json).map_err(D::Error::custom)?))
            }
            _ => Err(D::Error::custom(format!("event type `{}` is not a valid event", ev_type))),
        }
    }
}

impl<'de> de::Deserialize<'de> for AnyRoomEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = JsonValue::deserialize(deserializer)?;
        let ev_type: String = util::get_field(&json, "type")?;

        match ev_type.as_str() {
            "m.room.redaction" => {
                Ok(AnyRoomEvent::Redaction(from_json_value(json).map_err(D::Error::custom)?))
            }
            ev_type if AnyMessageEventContent::is_compatible(ev_type) => {
                Ok(AnyRoomEvent::Message(from_json_value(json).map_err(D::Error::custom)?))
            }
            ev_type if AnyStateEventContent::is_compatible(ev_type) => {
                Ok(AnyRoomEvent::State(from_json_value(json).map_err(D::Error::custom)?))
            }
            _ => Err(D::Error::custom(format!("event type `{}` is not a valid event", ev_type))),
        }
    }
}

impl<'de> de::Deserialize<'de> for AnyRoomEventStub {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = JsonValue::deserialize(deserializer)?;
        let ev_type: String = util::get_field(&json, "type")?;

        match ev_type.as_str() {
            "m.room.redaction" => {
                Ok(AnyRoomEventStub::Redaction(from_json_value(json).map_err(D::Error::custom)?))
            }
            ev_type if AnyMessageEventContent::is_compatible(ev_type) => {
                Ok(AnyRoomEventStub::Message(from_json_value(json).map_err(D::Error::custom)?))
            }
            ev_type if AnyStateEventContent::is_compatible(ev_type) => {
                Ok(AnyRoomEventStub::State(from_json_value(json).map_err(D::Error::custom)?))
            }
            _ => Err(D::Error::custom(format!("event type `{}` is not a valid event", ev_type))),
        }
    }
}
