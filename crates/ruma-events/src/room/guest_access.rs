//! Types for the *m.room.guest_access* event.

use ruma_events_macros::EventContent;
use ruma_serde::StringEnum;
use serde::{Deserialize, Serialize};

use crate::StateEvent;

/// Controls whether guest users are allowed to join rooms.
///
/// This event controls whether guest users are allowed to join rooms. If this event is absent,
/// servers should act as if it is present and has the value `GuestAccess::Forbidden`.
pub type GuestAccessEvent = StateEvent<GuestAccessEventContent>;

/// The payload for `GuestAccessEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.guest_access", kind = State)]
pub struct GuestAccessEventContent {
    /// A policy for guest user access to a room.
    pub guest_access: GuestAccess,
}

impl GuestAccessEventContent {
    /// Creates a new `GuestAccessEventContent` with the given policy.
    pub fn new(guest_access: GuestAccess) -> Self {
        Self { guest_access }
    }
}

/// A policy for guest user access to a room.
///
/// This type can hold an arbitrary string. To check for formats that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum GuestAccess {
    /// Guests are allowed to join the room.
    CanJoin,

    /// Guests are not allowed to join the room.
    Forbidden,

    #[doc(hidden)]
    _Custom(String),
}

impl GuestAccess {
    /// Creates a string slice from this `GuestAccess`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}
