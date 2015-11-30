//! Types for the *m.room.create* event.

use core::{Event, RoomEvent, StateEvent};

/// This is the first event in a room and cannot be changed. It acts as the root of all other
/// events.
pub struct CreateEvent<'a, 'b> {
    content: CreateEventContent<'a>,
    event_id: &'a str,
    prev_content: Option<CreateEventContent<'b>>,
    room_id: &'a str,
    user_id: &'a str,
}

impl<'a, 'b> Event<'a, CreateEventContent<'a>> for CreateEvent<'a, 'b> {
    fn content(&'a self) -> &'a CreateEventContent<'a> {
        &self.content
    }

    fn event_type(&self) -> &'static str {
        "m.room.create"
    }
}

impl<'a, 'b> RoomEvent<'a, CreateEventContent<'a>> for CreateEvent<'a, 'b> {
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

impl<'a, 'b> StateEvent<'a, 'b, CreateEventContent<'a>> for CreateEvent<'a, 'b> {}

/// The payload of a `CreateEvent`.
pub struct CreateEventContent<'a> {
    /// The `user_id` of the room creator. This is set by the homeserver.
    creator: &'a str,
}

