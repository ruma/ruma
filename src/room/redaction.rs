//! Types for the *m.room.avatar* event.

use core::EventType;

/// A redaction of an event.
pub struct RedactionEvent {
    content: RedactionEventContent,
    event_id: String,
    event_type: EventType,
    /// The ID of the event that was redacted.
    redacts: String,
    room_id: String,
    user_id: String,
}

/// The payload of a `RedactionEvent`.
pub struct RedactionEventContent {
    /// The reason for the redaction, if any.
    reason: Option<String>,
}
