//! Types for the *m.room.avatar* event.

use core::{Event, EventType, RoomEvent};

/// A redaction of an event.
pub struct RedactionEvent<'a> {
    content: RedactionEventContent<'a>,
    event_id: &'a str,
    /// The ID of the event that was redacted.
    redacts: &'a str,
    room_id: &'a str,
    user_id: &'a str,
}

impl<'a, 'b> Event<'a, RedactionEventContent<'a>> for RedactionEvent<'a> {
    fn content(&'a self) -> &'a RedactionEventContent<'a> {
        &self.content
    }

    fn event_type(&self) -> EventType {
        EventType::RoomRedaction
    }
}

impl<'a> RoomEvent<'a, RedactionEventContent<'a>> for RedactionEvent<'a> {
    fn event_id(&'a self) -> &'a str {
        &self.event_id
    }

    fn room_id(&'a self) -> &'a str {
        &self.room_id
    }

    fn user_id(&'a self) -> &'a str {
        &self.user_id
    }
}

/// The payload of a `RedactionEvent`.
pub struct RedactionEventContent<'a> {
    /// The reason for the redaction, if any.
    reason: Option<&'a str>,
}
