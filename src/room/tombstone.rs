//! Types for the *m.room.tombstone* event.

use ruma_identifiers::RoomId;
use serde::{Deserialize, Serialize};

state_event! {
    /// A state event signifying that a room has been upgraded to a different room version, and that
    /// clients should go there.
    pub struct TombstoneEvent(TombstoneEventContent) {}
}

/// The payload of an *m.room.tombstone* event.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TombstoneEventContent {
    /// A server-defined message.
    pub body: String,

    /// The new room the client should be visiting.
    pub replacement_room: RoomId,
}
