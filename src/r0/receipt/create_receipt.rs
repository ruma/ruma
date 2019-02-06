//! [POST /_matrix/client/r0/rooms/{roomId}/receipt/{receiptType}/{eventId}](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-rooms-roomid-receipt-receipttype-eventid)

use std::fmt::{Display, Error as FmtError, Formatter};

use ruma_api_macros::ruma_api;
use ruma_identifiers::{EventId, RoomId};
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata {
        description: "Send a receipt event to a room.",
        method: POST,
        name: "create_receipt",
        path: "/_matrix/client/r0/rooms/:room_id/receipt/:receipt_type/:event_id",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// The event ID to acknowledge up to.
        #[ruma_api(path)]
        pub event_id: EventId,
        /// The type of receipt to send.
        #[ruma_api(path)]
        pub receipt_type: ReceiptType,
        /// The room in which to send the event.
        #[ruma_api(path)]
        pub room_id: RoomId,
    }

    response {}
}

/// The type of receipt.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum ReceiptType {
    /// m.read
    #[serde(rename = "m.read")]
    Read,
}

impl Display for ReceiptType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        match *self {
            ReceiptType::Read => write!(f, "m.read"),
        }
    }
}
