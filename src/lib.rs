//! Crate ruma_events contains serializable types for the events in the [Matrix](https://matrix.org)
//! specification that can be shared by client and server code.

#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;

use std::fmt::{Display, Formatter, Error as FmtError};

use serde::{Deserialize, Serialize};

pub mod call;
pub mod presence;
pub mod receipt;
pub mod room;
pub mod stripped;
pub mod tag;
pub mod typing;

/// The type of an event.
#[derive(Debug, Deserialize, Serialize)]
pub enum EventType {
    CallAnswer,
    CallCandidates,
    CallHangup,
    CallInvite,
    Presence,
    Receipt,
    RoomAliases,
    RoomAvatar,
    RoomCanonicalAlias,
    RoomCreate,
    RoomGuestAccess,
    RoomHistoryVisibility,
    RoomJoinRules,
    RoomMember,
    RoomMessage,
    RoomName,
    RoomPowerLevels,
    RoomRedaction,
    RoomThirdPartyInvite,
    RoomTopic,
    Tag,
    Typing,
}

/// A basic event.
#[derive(Debug, Deserialize, Serialize)]
pub struct Event<T> where T: Deserialize + Serialize {
    pub content: T,
    #[serde(rename="type")]
    pub event_type: EventType,
}

/// An event within the context of a room.
#[derive(Debug, Deserialize, Serialize)]
pub struct RoomEvent<T> where T: Deserialize + Serialize {
    pub content: T,
    pub event_id: String,
    #[serde(rename="type")]
    pub event_type: EventType,
    pub room_id: String,
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
    pub prev_content: Option<T>,
    pub room_id: String,
    pub state_key: String,
    #[serde(rename="sender")]
    pub user_id: String,
}

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
        };

        write!(f, "{}", event_type_str)
    }
}
