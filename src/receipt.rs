//! Types for the *m.receipt* event.

use std::{collections::BTreeMap, time::SystemTime};

use ruma_events_macros::EphemeralRoomEventContent;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{Deserialize, Serialize};

/// Informs the client who has read a message specified by it's event id.
#[derive(Clone, Debug, Deserialize, Serialize, EphemeralRoomEventContent)]
#[ruma_event(type = "m.receipt")]
#[serde(transparent)]
pub struct ReceiptEventContent {
    /// The payload for `ReceiptEvent`.
    ///
    /// A mapping of event ID to a collection of receipts for this event ID. The event ID is the ID of
    /// the event being acknowledged and *not* an ID for the receipt itself.
    pub receipts: BTreeMap<EventId, Receipts>,
}

/// A collection of receipts.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Receipts {
    /// A collection of users who have sent *m.read* receipts for this event.
    #[serde(default, rename = "m.read")]
    pub read: Option<UserReceipts>,
}

/// A mapping of user ID to receipt.
///
/// The user ID is the entity who sent this receipt.
pub type UserReceipts = BTreeMap<UserId, Receipt>;

/// An acknowledgement of an event.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Receipt {
    /// The time when the receipt was sent.
    #[serde(
        with = "ruma_serde::time::opt_ms_since_unix_epoch",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub ts: Option<SystemTime>,
}
