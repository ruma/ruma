//! Types for the *m.room.name* event.

use core::EventType;

/// A human-friendly room name designed to be displayed to the end-user.
pub struct NameEvent {
    content: NameEventContent,
    event_id: String,
    event_type: EventType,
    prev_content: Option<NameEventContent>,
    room_id: String,
    state_key: String,
    user_id: String,
}

/// The payload of a `NameEvent`.
pub struct NameEventContent {
    /// The name of the room. This MUST NOT exceed 255 bytes.
    name: String,
}
