//! [POST /_matrix/client/r0/rooms/{roomId}/receipt/{receiptType}/{eventId}](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-rooms-roomid-receipt-receipttype-eventid)

use std::convert::TryFrom;

use ruma_api::ruma_api;
use ruma_identifiers::{EventId, RoomId};
use strum::{Display, EnumString};

ruma_api! {
    metadata: {
        description: "Send a receipt event to a room.",
        method: POST,
        name: "create_receipt",
        path: "/_matrix/client/r0/rooms/:room_id/receipt/:receipt_type/:event_id",
        rate_limited: true,
        requires_authentication: true,
    }

    request: {
        /// The room in which to send the event.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The type of receipt to send.
        #[ruma_api(path)]
        pub receipt_type: ReceiptType,

        /// The event ID to acknowledge up to.
        #[ruma_api(path)]
        pub event_id: &'a EventId,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room ID, receipt type and event ID.
    pub fn new(room_id: &'a RoomId, receipt_type: ReceiptType, event_id: &'a EventId) -> Self {
        Self { room_id, receipt_type, event_id }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}

/// The type of receipt.
#[derive(Clone, Copy, Debug, Display, EnumString)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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
