//! Types for the *m.room.join_rules* event.

use events::EventType;

/// Describes how users are allowed to join the room.
#[derive(Debug, Deserialize, Serialize)]
pub struct JoinRulesEvent {
    pub content: JoinRulesEventContent,
    pub event_id: String,
    pub event_type: EventType,
    pub prev_content: Option<JoinRulesEventContent>,
    pub room_id: String,
    pub state_key: String,
    pub user_id: String,
}

/// The payload of a `JoinRulesEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct JoinRulesEventContent {
    /// The type of rules used for users wishing to join this room.
    pub join_rule: JoinRule,
}

/// The rule used for users wishing to join this room.
#[derive(Debug, Deserialize, Serialize)]
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
