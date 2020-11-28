//! Types for the *m.room.join_rules* event.

use ruma_events_macros::StateEventContent;
use ruma_serde::StringEnum;
use serde::{Deserialize, Serialize};

use crate::StateEvent;

/// Describes how users are allowed to join the room.
pub type JoinRulesEvent = StateEvent<JoinRulesEventContent>;

/// The payload for `JoinRulesEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.join_rules")]
pub struct JoinRulesEventContent {
    /// The type of rules used for users wishing to join this room.
    #[ruma_event(skip_redaction)]
    pub join_rule: JoinRule,
}

impl JoinRulesEventContent {
    /// Creates a new `JoinRulesEventContent` with the given rule.
    pub fn new(join_rule: JoinRule) -> Self {
        Self { join_rule }
    }
}

/// The rule used for users wishing to join this room.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "lowercase")]
pub enum JoinRule {
    /// A user who wishes to join the room must first receive an invite to the room from someone
    /// already inside of the room.
    Invite,

    /// Reserved but not yet implemented by the Matrix specification.
    Knock,

    /// Reserved but not yet implemented by the Matrix specification.
    Private,

    /// Anyone can join the room without any prior action.
    Public,

    #[doc(hidden)]
    _Custom(String),
}
