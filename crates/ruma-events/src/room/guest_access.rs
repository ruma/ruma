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
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.guest_access", kind = State, state_key_type = EmptyStateKey)]
pub struct RoomGuestAccessEventContent {
    /// A policy for guest user access to a room.
    pub guest_access: GuestAccess,
}

impl RoomGuestAccessEventContent {
    /// Creates a new `RoomGuestAccessEventContent` with the given policy.
    pub fn new(guest_access: GuestAccess) -> Self {
        Self { guest_access }
    }
}

impl RoomGuestAccessEvent {
    /// Obtain the guest access policy, regardless of whether this event is redacted.
    pub fn guest_access(&self) -> &GuestAccess {
        match self {
            Self::Original(ev) => &ev.content.guest_access,
            Self::Redacted(_) => &GuestAccess::Forbidden,
        }
    }
}

impl SyncRoomGuestAccessEvent {
    /// Obtain the guest access policy, regardless of whether this event is redacted.
    pub fn guest_access(&self) -> &GuestAccess {
        match self {
            Self::Original(ev) => &ev.content.guest_access,
            Self::Redacted(_) => &GuestAccess::Forbidden,
        }
    }
}

/// A policy for guest user access to a room.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, StringEnum)]
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
