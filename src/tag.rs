//! Types for the *m.tag* event.

use std::collections::HashMap;

use core::{Event, EventType};

/// Informs the client of tags on a room.
pub struct TagEvent<'a> {
    /// The payload.
    content: TagEventContent<'a>,
}

impl<'a> Event<'a, TagEventContent<'a>> for TagEvent<'a> {
    fn content(&'a self) -> &'a TagEventContent<'a> {
        &self.content
    }

    fn event_type(&self) -> EventType {
        EventType::Tag
    }
}

/// The payload of a `TagEvent`.
pub struct TagEventContent<'a> {
    /// The list of user IDs typing in this room, if any.
    tags: &'a Tags<'a>,
}

/// A map of tag names to values.
pub type Tags<'a> = HashMap<&'a str, &'a str>;
