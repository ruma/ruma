//! Types for the [`m.room.tombstone`] event.
//!
//! [`m.room.tombstone`]: https://spec.matrix.org/latest/client-server-api/#mroomtombstone

use ruma_common::OwnedRoomId;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::EmptyStateKey;

/// The content of an `m.room.tombstone` event.
///
/// A state event signifying that a room has been upgraded to a different room version, and that
/// clients should go there.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(
    type = "m.room.tombstone",
    kind = State,
    state_key_type = EmptyStateKey,
)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RoomTombstoneEventContent {
    /// A server-defined message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// The new room the client should be visiting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement_room: Option<OwnedRoomId>,
}

impl RoomTombstoneEventContent {
    /// Creates a new `RoomTombstoneEventContent` with the given body and replacement room ID.
    pub fn new(body: String, replacement_room: OwnedRoomId) -> Self {
        Self { body: Some(body), replacement_room: Some(replacement_room) }
    }
}
