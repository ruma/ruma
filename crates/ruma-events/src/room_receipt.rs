//! Module for in-room read receipts.

use ruma_common::OwnedEventId;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of an in-room receipt event.
///
/// A mapping of event ID to a collection of receipts for this event ID. The event ID is the ID of
/// the event being acknowledged and *not* an ID for the receipt itself.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "m.room.receipt", kind = MessageLike)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomReceiptEventContent {
    /// The ID of the event we are acknowledging.
    pub event_id: OwnedEventId,
}
