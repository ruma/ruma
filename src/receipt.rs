//! Types for the *m.receipt* event.

use std::{collections::BTreeMap, time::SystemTime};

use ruma_events_macros::ruma_event;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{Deserialize, Serialize};

ruma_event! {
    /// Informs the client of new receipts.
    ReceiptEvent {
        kind: Event,
        event_type: "m.receipt",
        fields: {
            /// The unique identifier for the room associated with this event.
            ///
            /// `None` if the room is known through other means (such as this even being part of an
            /// event list scoped to a room in a `/sync` response)
            pub room_id: Option<RoomId>,
        },
        content_type_alias: {
            /// The payload for `ReceiptEvent`.
            ///
            /// A mapping of event ID to a collection of receipts for this event ID. The event ID is the ID of
            /// the event being acknowledged and *not* an ID for the receipt itself.
            BTreeMap<EventId, Receipts>
        },
    }
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
