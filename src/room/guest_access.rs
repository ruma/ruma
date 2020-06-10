//! Types for the *m.room.guest_access* event.

use ruma_events_macros::StateEventContent;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::StateEvent;

/// Controls whether guest users are allowed to join rooms.
///
/// This event controls whether guest users are allowed to join rooms. If this event is absent,
/// servers should act as if it is present and has the value `GuestAccess::Forbidden`.
pub type GuestAccessEvent = StateEvent<GuestAccessEventContent>;

/// The payload for `GuestAccessEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[ruma_event(type = "m.room.guest_access")]
pub struct GuestAccessEventContent {
    /// A policy for guest user access to a room.
    pub guest_access: GuestAccess,
}

/// A policy for guest user access to a room.
#[derive(Clone, Copy, Debug, PartialEq, Display, EnumString, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum GuestAccess {
    /// Guests are allowed to join the room.
    CanJoin,

    /// Guests are not allowed to join the room.
    Forbidden,
}
