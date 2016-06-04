//! Types for the *m.room.create* event.

use core::EventType;

/// This is the first event in a room and cannot be changed. It acts as the root of all other
/// events.
pub struct CreateEvent {
    content: CreateEventContent,
    event_id: String,
    event_type: EventType,
    prev_content: Option<CreateEventContent>,
    room_id: String,
    state_key: String,
    user_id: String,
}

/// The payload of a `CreateEvent`.
pub struct CreateEventContent {
    /// The `user_id` of the room creator. This is set by the homeserver.
    creator: String,
}
