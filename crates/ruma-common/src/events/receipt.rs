//! Types for the [`m.receipt`] event.
//!
//! [`m.receipt`]: https://spec.matrix.org/v1.2/client-server-api/#mreceipt

use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use ruma_common::{receipt::ReceiptType, EventId, MilliSecondsSinceUnixEpoch, UserId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of an `m.receipt` event.
///
/// A mapping of event ID to a collection of receipts for this event ID. The event ID is the ID of
/// the event being acknowledged and *not* an ID for the receipt itself.
///
/// Informs the client who has read a message specified by it's event id.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[allow(clippy::exhaustive_structs)]
#[ruma_event(type = "m.receipt", kind = EphemeralRoom)]
pub struct ReceiptEventContent(pub BTreeMap<Box<EventId>, Receipts>);

impl Deref for ReceiptEventContent {
    type Target = BTreeMap<Box<EventId>, Receipts>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ReceiptEventContent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A collection of receipts.
pub type Receipts = BTreeMap<ReceiptType, UserReceipts>;

/// A mapping of user ID to receipt.
///
/// The user ID is the entity who sent this receipt.
pub type UserReceipts = BTreeMap<Box<UserId>, Receipt>;

/// An acknowledgement of an event.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Receipt {
    /// The time when the receipt was sent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts: Option<MilliSecondsSinceUnixEpoch>,
}

impl Receipt {
    /// Creates a new `Receipt` with the given timestamp.
    ///
    /// To create an empty receipt instead, use [`Receipt::default`].
    pub fn new(ts: MilliSecondsSinceUnixEpoch) -> Self {
        Self { ts: Some(ts) }
    }
}
