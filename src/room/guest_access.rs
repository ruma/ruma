//! Types for the *m.room.guest_access* event.

use StateEvent;

/// Controls whether guest users are allowed to join rooms.
///
/// This event controls whether guest users are allowed to join rooms. If this event is absent,
/// servers should act as if it is present and has the value `GuestAccess::Forbidden`.
pub type GuestAccessEvent = StateEvent<GuestAccessEventContent, ()>;

/// The payload of a `GuestAccessEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct GuestAccessEventContent {
    /// A policy for guest user access to a room.
    pub guest_access: GuestAccess,
}

/// A policy for guest user access to a room.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub enum GuestAccess {
    /// Guests are allowed to join the room.
    #[serde(rename="can_join")]
    CanJoin,

    /// Guests are not allowed to join the room.
    #[serde(rename="forbidden")]
    Forbidden,
}

impl_enum! {
    GuestAccess {
        CanJoin => "can_join",
        Forbidden => "forbidden",
    }
}
