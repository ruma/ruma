//! Types for the *m.room.join_rules* event.

use core::EventType;

/// Describes how users are allowed to join the room.
pub struct JoinRulesEvent {
    content: JoinRulesEventContent,
    event_id: String,
    event_type: EventType,
    prev_content: Option<JoinRulesEventContent>,
    room_id: String,
    state_key: String,
    user_id: String,
}

/// The payload of a `JoinRulesEvent`.
pub struct JoinRulesEventContent {
    /// The type of rules used for users wishing to join this room.
    join_rule: JoinRule,
}

/// The rule used for users wishing to join this room.
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
}
