//! Types for the *m.typing* event.

use core::EventType;

/// Informs the client of the list of users currently typing.
#[derive(Debug, Deserialize, Serialize)]
pub struct TypingEvent {
    /// The payload.
    pub content: TypingEventContent,
    pub event_type: EventType,
    /// The ID of the room associated with this event.
    pub room_id: String,
}

/// The payload of a `TypingEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct TypingEventContent {
    /// The list of user IDs typing in this room, if any.
    pub user_ids: Vec<String>,
}
