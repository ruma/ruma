//! Types for the *m.room.join_rules* event.

use js_int::UInt;
use serde::{Deserialize, Serialize};

state_event! {
    /// Describes how users are allowed to join the room.
    pub struct JoinRulesEvent(JoinRulesEventContent) {}
}

/// The payload of a `JoinRulesEvent`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct JoinRulesEventContent {
    /// The type of rules used for users wishing to join this room.
    pub join_rule: JoinRule,
}

/// The rule used for users wishing to join this room.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum JoinRule {
    /// A user who wishes to join the room must first receive an invite to the room from someone
    /// already inside of the room.
    #[serde(rename = "invite")]
    Invite,

    /// Reserved but not yet implemented by the Matrix specification.
    #[serde(rename = "knock")]
    Knock,

    /// Reserved but not yet implemented by the Matrix specification.
    #[serde(rename = "private")]
    Private,

    /// Anyone can join the room without any prior action.
    #[serde(rename = "public")]
    Public,

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to ruma-events.
    #[doc(hidden)]
    #[serde(skip)]
    __Nonexhaustive,
}

impl_enum! {
    JoinRule {
        Invite => "invite",
        Knock => "knock",
        Private => "private",
        Public => "public",
    }
}
