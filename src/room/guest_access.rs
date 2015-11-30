//! Types for the *m.room.guest_access* event.

use core::{Event, EventType, RoomEvent, StateEvent};

/// Controls whether guest users are allowed to join rooms.
///
/// This event controls whether guest users are allowed to join rooms. If this event is absent,
/// servers should act as if it is present and has the value `GuestAccess::Forbidden`.
pub struct GuestAccessEvent<'a, 'b> {
    content: GuestAccessEventContent<'a>,
    event_id: &'a str,
    prev_content: Option<GuestAccessEventContent<'b>>,
    room_id: &'a str,
    user_id: &'a str,
}

impl<'a, 'b> Event<'a, GuestAccessEventContent<'a>> for GuestAccessEvent<'a, 'b> {
    fn content(&'a self) -> &'a GuestAccessEventContent<'a> {
        &self.content
    }

    fn event_type(&self) -> EventType {
        EventType::RoomGuestAccess
    }
}

impl<'a, 'b> RoomEvent<'a, GuestAccessEventContent<'a>> for GuestAccessEvent<'a, 'b> {
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

impl<'a, 'b> StateEvent<'a, 'b, GuestAccessEventContent<'a>> for GuestAccessEvent<'a, 'b> {
    fn prev_content(&'a self) -> Option<&'b GuestAccessEventContent> {
        match self.prev_content {
            Some(ref prev_content) => Some(prev_content),
            None => None,
        }
    }

    fn state_key(&self) -> &'a str {
        ""
    }
}

/// The payload of a `GuestAccessEvent`.
pub struct GuestAccessEventContent<'a> {
    guest_access: &'a GuestAccess,
}

/// A policy for guest user access to a room.
pub enum GuestAccess {
    /// Guests are allowed to join the room.
    CanJoin,
    /// Guests are not allowed to join the room.
    Forbidden,
}
