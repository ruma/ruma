//! Types for the [`m.room.guest_access`] event.
//!
//! [`m.room.guest_access`]: https://spec.matrix.org/latest/client-server-api/#mroomguest_access

use ruma_common::serde::StringEnum;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{EmptyStateKey, PrivOwnedStr};

/// The content of an `m.room.guest_access` event.
///
/// Controls whether guest users are allowed to join rooms.
///
/// This event controls whether guest users are allowed to join rooms. If this event is absent,
/// servers should act as if it is present and has the value `GuestAccess::Forbidden`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.room.guest_access", kind = State, state_key_type = EmptyStateKey)]
pub struct RoomGuestAccessEventContent {
    /// A policy for guest user access to a room.
    #[serde(default = "default_guest_access", skip_serializing_if = "is_default_guest_access")]
    pub guest_access: GuestAccess,
}

impl RoomGuestAccessEventContent {
    /// Creates a new `RoomGuestAccessEventContent` with the given policy.
    pub fn new(guest_access: GuestAccess) -> Self {
        Self { guest_access }
    }
}

/// A policy for guest user access to a room.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum GuestAccess {
    /// Guests are allowed to join the room.
    CanJoin,

    /// Guests are not allowed to join the room.
    Forbidden,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The default guest access when the state is unset.
fn default_guest_access() -> GuestAccess {
    GuestAccess::Forbidden
}

/// Whether the given guest access matches the default when the state is unset.
fn is_default_guest_access(access: &GuestAccess) -> bool {
    matches!(access, GuestAccess::Forbidden)
}
