//! Types for the *m.receipt* event.

use std::collections::HashMap;

use ruma_identifiers::{EventId, RoomId, UserId};

event! {
    /// Informs the client of new receipts.
    pub struct ReceiptEvent(ReceiptEventContent) {
        /// The unique identifier for the room associated with this event.
        ///
        /// This can be `None` if the event came from a context where there is
        /// no ambiguity which room it belongs to, like a `/sync` response for example.
        #[serde(skip_serializing_if="Option::is_none")]
        pub room_id: Option<RoomId>
    }
}

/// The payload of a `ReceiptEvent`.
///
/// A mapping of event ID to a collection of receipts for this event ID. The event ID is the ID of
/// the event being acknowledged and *not* an ID for the receipt itself.
pub type ReceiptEventContent = HashMap<EventId, Receipts>;

/// A collection of receipts.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Receipts {
    /// A collection of users who have sent *m.read* receipts for this event.
    #[serde(rename = "m.read")]
    #[serde(default)]
    pub m_read: UserReceipts,
}

/// A mapping of user ID to receipt.
///
/// The user ID is the entity who sent this receipt.
pub type UserReceipts = HashMap<UserId, Receipt>;

/// An acknowledgement of an event.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Receipt {
    /// The timestamp the receipt was sent at.
    pub ts: u64,
}
