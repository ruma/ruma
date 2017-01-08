//! Endpoints for event receipts.

/// [POST /_matrix/client/r0/rooms/{roomId}/receipt/{receiptType}/{eventId}](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-rooms-roomid-receipt-receipttype-eventid)
pub mod create_receipt {
    use ruma_identifiers::{EventId, RoomId};

    use std::fmt::{Display, Error as FmtError, Formatter};

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    /// This endpoint's path parameters.
    #[derive(Clone, Debug)]
    pub struct PathParams {
        /// The event ID to acknowledge up to.
        pub event_id: EventId,
        /// The type of receipt to send.
        pub receipt_type: ReceiptType,
        /// The room in which to send the event.
        pub room_id: RoomId,
    }

    /// The type of receipt.
    #[derive(Clone, Copy, Debug)]
    pub enum ReceiptType {
        /// m.read
        Read,
    }

    /// This API endpoint's response.
    #[derive(Clone, Copy, Debug, Deserialize, Serialize)]
    pub struct Response;

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = Response;

        fn method() -> ::Method {
            ::Method::Post
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/rooms/{}/receipt/{}/{}",
                params.room_id,
                params.receipt_type,
                params.event_id
            )
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/rooms/:room_id/receipt/:receipt_type/:event_id"
        }

        fn name() -> &'static str {
            "create_receipt"
        }

        fn description() -> &'static str {
            "Send a receipt event to a room."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            true
        }
    }

    impl Display for ReceiptType {
        fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
            match *self {
                ReceiptType::Read => write!(f, "m.read"),
            }
        }
    }
}
