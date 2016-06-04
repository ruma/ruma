//! Types for the *m.room.guest_access* event.

use core::EventType;

/// Controls whether guest users are allowed to join rooms.
///
/// This event controls whether guest users are allowed to join rooms. If this event is absent,
/// servers should act as if it is present and has the value `GuestAccess::Forbidden`.
pub struct GuestAccessEvent {
    content: GuestAccessEventContent,
    event_id: String,
    event_type: EventType,
    prev_content: Option<GuestAccessEventContent>,
    room_id: String,
    state_key: String,
    user_id: String,
}

/// The payload of a `GuestAccessEvent`.
pub struct GuestAccessEventContent {
    guest_access: GuestAccess,
}

/// A policy for guest user access to a room.
pub enum GuestAccess {
    /// Guests are allowed to join the room.
    CanJoin,
    /// Guests are not allowed to join the room.
    Forbidden,
}
