//! Types for the *m.fully_read* event.

use ruma_identifiers::{EventId, RoomId};
use serde::{Deserialize, Serialize};

event! {
    /// The current location of the user's read marker in a room.
    ///
    /// This event appears in the user's room account data for the room the marker is applicable
    /// for.
    pub struct FullyReadEvent(FullyReadEventContent) {
        /// The unique identifier for the room associated with this event.
        pub room_id: RoomId
    }
}

/// The payload of a `FullyReadEvent`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FullyReadEventContent {
    /// The event the user's read marker is located at in the room.
    pub event_id: EventId,
}
