//! Types for the *m.room.aliases* event.

use core::{Event, RoomEvent, StateEvent};

/// Informs the room about what room aliases it has been given.
pub struct AliasesEvent<'a, 'b> {
    content: AliasesEventContent<'a>,
    event_id: &'a str,
    prev_content: Option<AliasesEventContent<'b>>,
    room_id: &'a str,
    /// The homeserver domain which owns these room aliases.
    state_key: &'a str,
    user_id: &'a str,
}

impl<'a, 'b> Event<'a, AliasesEventContent<'a>> for AliasesEvent<'a, 'b> {
    fn content(&'a self) -> &'a AliasesEventContent<'a> {
        &self.content
    }

    fn event_type(&self) -> &'static str {
        "m.room.aliases"
    }
}

impl<'a, 'b> RoomEvent<'a, AliasesEventContent<'a>> for AliasesEvent<'a, 'b> {
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

impl<'a, 'b> StateEvent<'a, 'b, AliasesEventContent<'a>> for AliasesEvent<'a, 'b> {
    fn prev_content(&'a self) -> Option<&'b AliasesEventContent> {
        match self.prev_content {
            Some(ref prev_content) => Some(prev_content),
            None => None,
        }
    }

    fn state_key(&self) -> &'a str {
        &self.state_key
    }
}

/// The payload of an `AliasesEvent`.
pub struct AliasesEventContent<'a> {
    /// A list of room aliases.
    aliases: &'a[&'a str],
}
