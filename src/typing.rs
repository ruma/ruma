//! Events within the *m.typing* namespace.

use core::Event;

/// Informs the client of the list of users currently typing.
pub struct Typing<'a> {
    /// The payload.
    content: TypingContent<'a>,
    /// The ID of the room associated with this event.
    room_id: &'a str,
}

impl<'a> Event<'a, TypingContent<'a>> for Typing<'a> {
    fn content(&'a self) -> &'a TypingContent<'a> {
        &self.content
    }

    fn event_type(&self) -> &'static str {
        "m.typing"
    }
}

/// The payload of a `Typing` event.
pub struct TypingContent<'a> {
    /// The list of user IDs typing in this room, if any.
    user_ids: &'a[&'a str],
}
