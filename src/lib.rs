//! Crate ruma_events contains serializable types for the events in the [Matrix](https://matrix.org)
//! specification that can be shared by client and server code.

#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;
extern crate serde_json;

use std::fmt::{Display, Formatter, Error as FmtError};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod call;
pub mod presence;
pub mod receipt;
pub mod room;
pub mod stripped;
pub mod tag;
pub mod typing;

/// A basic event.
#[derive(Debug, Deserialize, Serialize)]
pub struct Event<T> where T: Deserialize + Serialize {
    pub content: T,
    #[serde(rename="type")]
    pub event_type: EventType,
    /// Extra key-value pairs to be mixed into the top-level JSON representation of the event.
    pub extra_content: Option<Value>,
}

/// A type that represents a kind of Matrix event.
///
/// The event kinds are basic events, room events, and state events.
/// This trait can be useful to constrain a generic parameter that must be a Matrix event.
pub trait EventKind: Deserialize + Serialize {}

/// The type of an event.
#[derive(Debug, Deserialize, Serialize)]
pub enum EventType {
    /// m.call.answer
    CallAnswer,
    /// m.call.candidates
    CallCandidates,
    /// m.call.hangup
    CallHangup,
    /// m.call.invite
    CallInvite,
    /// m.presence
    Presence,
    /// m.receipt
    Receipt,
    /// m.room.aliases
    RoomAliases,
    /// m.room.avatar
    RoomAvatar,
    /// m.room.canonical_alias
    RoomCanonicalAlias,
    /// m.room.create
    RoomCreate,
    /// m.room.guest_access
    RoomGuestAccess,
    /// m.room.history_visibility
    RoomHistoryVisibility,
    /// m.room.join_rules
    RoomJoinRules,
    /// m.room.member
    RoomMember,
    /// m.room.message
    RoomMessage,
    /// m.room.name
    RoomName,
    /// m.room.power_levels
    RoomPowerLevels,
    /// m.room.redaction
    RoomRedaction,
    /// m.room.third_party_invite
    RoomThirdPartyInvite,
    /// m.room.topic
    RoomTopic,
    /// m.tag
    Tag,
    /// m.typing
    Typing,
    /// Any event that is not part of the specification.
    Custom(String),
}

/// An event within the context of a room.
#[derive(Debug, Deserialize, Serialize)]
pub struct RoomEvent<T> where T: Deserialize + Serialize {
    pub content: T,
    pub event_id: String,
    /// Extra key-value pairs to be mixed into the top-level JSON representation of the event.
    pub extra_content: Option<Value>,
    #[serde(rename="type")]
    pub event_type: EventType,
    pub room_id: String,
    pub unsigned: Option<Value>,
    #[serde(rename="sender")]
    pub user_id: String,
}

/// An event that describes persistent state about a room.
#[derive(Debug, Deserialize, Serialize)]
pub struct StateEvent<T> where T: Deserialize + Serialize {
    pub content: T,
    pub event_id: String,
    #[serde(rename="type")]
    pub event_type: EventType,
    /// Extra key-value pairs to be mixed into the top-level JSON representation of the event.
    pub extra_content: Option<Value>,
    pub prev_content: Option<T>,
    pub room_id: String,
    pub state_key: String,
    pub unsigned: Option<Value>,
    #[serde(rename="sender")]
    pub user_id: String,
}

impl<T> EventKind for Event<T> where T: Deserialize + Serialize {}
impl<T> EventKind for RoomEvent<T> where T: Deserialize + Serialize {}
impl<T> EventKind for StateEvent<T> where T: Deserialize + Serialize {}

impl Display for EventType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        let event_type_str = match *self {
            EventType::CallAnswer => "m.call.answer",
            EventType::CallCandidates => "m.call.candidates",
            EventType::CallHangup => "m.call.hangup",
            EventType::CallInvite => "m.call.invite",
            EventType::Presence => "m.presence",
            EventType::Receipt => "m.receipt",
            EventType::RoomAliases => "m.room.aliases",
            EventType::RoomAvatar => "m.room.avatar",
            EventType::RoomCanonicalAlias => "m.room.canonical_alias",
            EventType::RoomCreate => "m.room.create",
            EventType::RoomGuestAccess => "m.room.guest_access",
            EventType::RoomHistoryVisibility => "m.room.history_visibility",
            EventType::RoomJoinRules => "m.room.join_rules",
            EventType::RoomMember => "m.room.member",
            EventType::RoomMessage => "m.room.message",
            EventType::RoomName => "m.room.name",
            EventType::RoomPowerLevels => "m.room.power_levels",
            EventType::RoomRedaction => "m.room.redaction",
            EventType::RoomThirdPartyInvite => "m.room.third_party_invite",
            EventType::RoomTopic => "m.room.topic",
            EventType::Tag => "m.tag",
            EventType::Typing => "m.typing",
            EventType::Custom(ref event_type) => event_type,
        };

        write!(f, "{}", event_type_str)
    }
}
