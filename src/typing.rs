//! Types for the *m.typing* event.

use ruma_identifiers::{EventId, RoomId};

use Event;

/// Informs the client of the list of users currently typing.
pub type TypingEvent = Event<TypingEventContent, TypingEventExtraContent>;

/// The payload of a `TypingEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct TypingEventContent {
    /// The list of user IDs typing in this room, if any.
    pub user_ids: Vec<EventId>,
}

/// Extra content for a `TypingEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct TypingEventExtraContent {
    /// The unique identifier for the room associated with this event.
    pub room_id: RoomId,
}
