//! Types for the *m.room.name* event.

use core::{Event, EventType, RoomEvent, StateEvent};

/// A human-friendly room name designed to be displayed to the end-user.
pub struct NameEvent<'a, 'b> {
    content: NameEventContent<'a>,
    event_id: &'a str,
    prev_content: Option<NameEventContent<'b>>,
    room_id: &'a str,
    user_id: &'a str,
}

impl<'a, 'b> Event<'a, NameEventContent<'a>> for NameEvent<'a, 'b> {
    fn content(&'a self) -> &'a NameEventContent<'a> {
        &self.content
    }

    fn event_type(&self) -> EventType {
        EventType::RoomName
    }
}

impl<'a, 'b> RoomEvent<'a, NameEventContent<'a>> for NameEvent<'a, 'b> {
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

impl<'a, 'b> StateEvent<'a, 'b, NameEventContent<'a>> for NameEvent<'a, 'b> {
    fn prev_content(&'a self) -> Option<&'b NameEventContent> {
        match self.prev_content {
            Some(ref prev_content) => Some(prev_content),
            None => None,
        }
    }
}

/// The payload of a `NameEvent`.
pub struct NameEventContent<'a> {
    /// The name of the room. This MUST NOT exceed 255 bytes.
    name: &'a str,
}
