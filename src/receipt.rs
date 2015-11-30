//! Types for the *m.receipt* event.

use std::collections::HashMap;

use core::{Event, EventType};

/// Informs the client of new receipts.
pub struct ReceiptEvent<'a> {
    content: ReceiptEventContent<'a>,
    room_id: &'a str,
}

impl<'a> Event<'a, ReceiptEventContent<'a>> for ReceiptEvent<'a> {
    fn content(&'a self) -> &'a ReceiptEventContent {
        &self.content
    }

    fn event_type(&self) -> EventType {
        EventType::Receipt
    }
}

/// The payload of a `ReceiptEvent`.
///
/// A mapping of event ID to a collection of receipts for this event ID. The event ID is the ID of
/// the event being acknowledged and *not* an ID for the receipt itself.
pub type ReceiptEventContent<'a> = HashMap<&'a str, Receipts<'a>>;

/// A collection of receipts.
pub struct Receipts<'a> {
    /// A collection of users who have sent *m.read* receipts for this event.
    m_read: UserReceipts<'a>,
}

/// A mapping of user ID to receipt.
///
/// The user ID is the entity who sent this receipt.
pub type UserReceipts<'a> = HashMap<&'a str, Receipt>;

/// An acknowledgement of an event.
pub struct Receipt {
    /// The timestamp the receipt was sent at.
    ts: u64,
}
