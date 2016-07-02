//! Types for the *m.typing* event.

use Event;

/// Informs the client of the list of users currently typing.
pub type TypingEvent = Event<TypingEventContent, TypingEventExtraContent>;

/// The payload of a `TypingEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct TypingEventContent {
    /// The list of user IDs typing in this room, if any.
    pub user_ids: Vec<String>,
}

/// Extra content for a `TypingEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct TypingEventExtraContent {
    /// The unique identifier for the room associated with this event.
    pub room_id: String,
}
