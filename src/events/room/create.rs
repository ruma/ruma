//! Types for the *m.room.create* event.

use events::EventType;

/// This is the first event in a room and cannot be changed. It acts as the root of all other
/// events.
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateEvent {
    pub content: CreateEventContent,
    pub event_id: String,
    pub event_type: EventType,
    pub prev_content: Option<CreateEventContent>,
    pub room_id: String,
    pub state_key: String,
    pub user_id: String,
}

/// The payload of a `CreateEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateEventContent {
    /// The `user_id` of the room creator. This is set by the homeserver.
    pub creator: String,
}
