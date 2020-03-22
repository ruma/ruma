//! [POST /_matrix/client/r0/rooms/{roomId}/receipt/{receiptType}/{eventId}](https://matrix.org/docs/spec/client_server/r0.4.0.html#post-matrix-client-r0-rooms-roomid-receipt-receipttype-eventid)

use std::convert::TryFrom;

use ruma_api::ruma_api;
use ruma_identifiers::{EventId, RoomId};
use strum::{Display, EnumString};

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

    error: crate::Error
}

/// The type of receipt.
#[derive(Clone, Copy, Debug, Display, EnumString)]
pub enum ReceiptType {
    /// m.read
    #[strum(serialize = "m.read")]
    Read,
}

impl TryFrom<&'_ str> for ReceiptType {
    type Error = strum::ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}
