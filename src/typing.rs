//! Types for the *m.typing* event.

use core::{Event, EventType};

/// Informs the client of the list of users currently typing.
pub struct TypingEvent<'a> {
    /// The payload.
    content: TypingEventContent<'a>,
    /// The ID of the room associated with this event.
    room_id: &'a str,
}

impl<'a> Event<'a, TypingEventContent<'a>> for TypingEvent<'a> {
    fn content(&'a self) -> &'a TypingEventContent<'a> {
        &self.content
    }

    fn event_type(&self) -> EventType {
        EventType::Typing
    }
}

/// The payload of a `TypingEvent`.
pub struct TypingEventContent<'a> {
    /// The list of user IDs typing in this room, if any.
    user_ids: &'a[&'a str],
}
