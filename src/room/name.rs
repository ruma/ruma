//! Types for the *m.room.name* event.

use core::EventType;

/// A human-friendly room name designed to be displayed to the end-user.
#[derive(Debug, Deserialize, Serialize)]
pub struct NameEvent {
    pub content: NameEventContent,
    pub event_id: String,
    pub event_type: EventType,
    pub prev_content: Option<NameEventContent>,
    pub room_id: String,
    pub state_key: String,
    pub user_id: String,
}

/// The payload of a `NameEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct NameEventContent {
    /// The name of the room. This MUST NOT exceed 255 bytes.
    pub name: String,
}
