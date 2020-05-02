//! Types for the *m.room.guest_access* event.

use ruma_events_macros::ruma_event;
use serde::{Deserialize, Serialize};

ruma_event! {
    /// Controls whether guest users are allowed to join rooms.
    ///
    /// This event controls whether guest users are allowed to join rooms. If this event is absent,
    /// servers should act as if it is present and has the value `GuestAccess::Forbidden`.
    GuestAccessEvent {
        kind: StateEvent,
        event_type: "m.room.guest_access",
        content: {
            /// A policy for guest user access to a room.
            pub guest_access: GuestAccess,
        },
    }
}

/// A policy for guest user access to a room.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum GuestAccess {
    /// Guests are allowed to join the room.
    CanJoin,

    /// Guests are not allowed to join the room.
    Forbidden,
}

impl_enum! {
    GuestAccess {
        CanJoin => "can_join",
        Forbidden => "forbidden",
    }
}
