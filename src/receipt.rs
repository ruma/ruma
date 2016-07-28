//! Types for the *m.receipt* event.

use std::collections::HashMap;

use ruma_identifiers::RoomId;

use Event;

/// Informs the client of new receipts.
pub type ReceiptEvent = Event<ReceiptEventContent, ReceiptEventExtraContent>;

/// The payload of a `ReceiptEvent`.
///
/// A mapping of event ID to a collection of receipts for this event ID. The event ID is the ID of
/// the event being acknowledged and *not* an ID for the receipt itself.
pub type ReceiptEventContent = HashMap<String, Receipts>;

/// A collection of receipts.
#[derive(Debug, Deserialize, Serialize)]
pub struct Receipts {
    /// A collection of users who have sent *m.read* receipts for this event.
    pub m_read: UserReceipts,
}

/// A mapping of user ID to receipt.
///
/// The user ID is the entity who sent this receipt.
pub type UserReceipts = HashMap<String, Receipt>;

/// An acknowledgement of an event.
#[derive(Debug, Deserialize, Serialize)]
pub struct Receipt {
    /// The timestamp the receipt was sent at.
    pub ts: u64,
}

/// Extra content for a `PresenceEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct ReceiptEventExtraContent {
    /// The unique identifier for the room associated with this event.
    pub room_id: RoomId,
}
