//! Types for the *m.fully_read* event.

use ruma_events_macros::BasicEventContent;
use ruma_identifiers::EventId;
use serde::{Deserialize, Serialize};

use crate::BasicEvent;

/// The current location of the user's read marker in a room.
///
/// This event appears in the user's room account data for the room the marker is applicable
/// for.
pub type FullyReadEvent = BasicEvent<FullyReadEventContent>;

/// The payload for `FullyReadEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.fully_read")]
pub struct FullyReadEventContent {
    /// The event the user's read marker is located at in the room.
    pub event_id: EventId,
}
