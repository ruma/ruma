//! Types for the *m.room.guest_access* event.

use events::EventType;

/// Controls whether guest users are allowed to join rooms.
///
/// This event controls whether guest users are allowed to join rooms. If this event is absent,
/// servers should act as if it is present and has the value `GuestAccess::Forbidden`.
#[derive(Debug, Deserialize, Serialize)]
pub struct GuestAccessEvent {
    pub content: GuestAccessEventContent,
    pub event_id: String,
    pub event_type: EventType,
    pub prev_content: Option<GuestAccessEventContent>,
    pub room_id: String,
    pub state_key: String,
    pub user_id: String,
}

/// The payload of a `GuestAccessEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct GuestAccessEventContent {
    pub guest_access: GuestAccess,
}

/// A policy for guest user access to a room.
#[derive(Debug, Deserialize, Serialize)]
pub enum GuestAccess {
    /// Guests are allowed to join the room.
    CanJoin,
    /// Guests are not allowed to join the room.
    Forbidden,
}
