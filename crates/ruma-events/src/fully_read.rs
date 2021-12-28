//! Types for the [`m.fully_read`] event.
//!
//! [`m.fully_read`]: https://spec.matrix.org/v1.1/client-server-api/#mfully_read

use ruma_events_macros::EventContent;
use ruma_identifiers::EventId;
use serde::{Deserialize, Serialize};

/// The content of an `m.fully_read` event.
///
/// The current location of the user's read marker in a room.
///
/// This event appears in the user's room account data for the room the marker is applicable for.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.fully_read", kind = RoomAccountData)]
pub struct FullyReadEventContent {
    /// The event the user's read marker is located at in the room.
    pub event_id: Box<EventId>,
}

impl FullyReadEventContent {
    /// Creates a new `FullyReadEventContent` with the given event ID.
    pub fn new(event_id: Box<EventId>) -> Self {
        Self { event_id }
    }
}
