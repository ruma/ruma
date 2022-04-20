//! Types for the [`m.room.tombstone`] event.
//!
//! [`m.room.tombstone`]: https://spec.matrix.org/v1.2/client-server-api/#mroomtombstone

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{events::EmptyStateKey, OwnedRoomId};

/// The content of an `m.room.tombstone` event.
///
/// A state event signifying that a room has been upgraded to a different room version, and that
/// clients should go there.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "m.room.tombstone", kind = State, state_key_type = EmptyStateKey)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomTombstoneEventContent {
    /// A server-defined message.
    ///
    /// If you activate the `compat` feature, this field being absent in JSON will result in an
    /// empty string here during deserialization.
    #[cfg_attr(feature = "compat", serde(default))]
    pub body: String,

    /// The new room the client should be visiting.
    pub replacement_room: OwnedRoomId,
}

impl RoomTombstoneEventContent {
    /// Creates a new `RoomTombstoneEventContent` with the given body and replacement room ID.
    pub fn new(body: String, replacement_room: OwnedRoomId) -> Self {
        Self { body, replacement_room }
    }
}
