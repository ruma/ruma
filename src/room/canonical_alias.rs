//! Types for the *m.room.canonical_alias* event.

use core::{Event, EventType, RoomEvent, StateEvent};

/// Informs the room as to which alias is the canonical one.
pub struct CanonicalAliasEvent<'a, 'b> {
    content: CanonicalAliasEventContent<'a>,
    event_id: &'a str,
    prev_content: Option<CanonicalAliasEventContent<'b>>,
    room_id: &'a str,
    user_id: &'a str,
}

impl<'a, 'b> Event<'a, CanonicalAliasEventContent<'a>> for CanonicalAliasEvent<'a, 'b> {
    fn content(&'a self) -> &'a CanonicalAliasEventContent<'a> {
        &self.content
    }

    fn event_type(&self) -> EventType {
        EventType::RoomCanonicalAlias
    }
}

impl<'a, 'b> RoomEvent<'a, CanonicalAliasEventContent<'a>> for CanonicalAliasEvent<'a, 'b> {
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

impl<'a, 'b> StateEvent<'a, 'b, CanonicalAliasEventContent<'a>> for CanonicalAliasEvent<'a, 'b> {
    fn prev_content(&'a self) -> Option<&'b CanonicalAliasEventContent> {
        match self.prev_content {
            Some(ref prev_content) => Some(prev_content),
            None => None,
        }
    }
}

/// The payload of a `CanonicalAliasEvent`.
pub struct CanonicalAliasEventContent<'a> {
    /// The canonical alias.
    alias: &'a str,
}

