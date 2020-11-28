//! [POST /_matrix/client/r0/rooms/{roomId}/receipt/{receiptType}/{eventId}](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-rooms-roomid-receipt-receipttype-eventid)

use ruma_api::ruma_api;
use ruma_identifiers::{EventId, RoomId};
use ruma_serde::{AsRefStr, DisplayAsRefStr, FromString};

ruma_api! {
    metadata: {
        description: "Send a receipt event to a room.",
        method: POST,
        name: "create_receipt",
        path: "/_matrix/client/r0/rooms/:room_id/receipt/:receipt_type/:event_id",
        rate_limited: true,
        authentication: AccessToken,
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
#[derive(Clone, Debug, AsRefStr, DisplayAsRefStr, FromString)]
pub enum ReceiptType {
    /// m.read
    #[ruma_enum(rename = "m.read")]
    Read,

    #[doc(hidden)]
    _Custom(String),
}
