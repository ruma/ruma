//! `POST /_matrix/client/*/rooms/{roomId}/receipt/{receiptType}/{eventId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3roomsroomidreceiptreceipttypeeventid

    use ruma_common::{api::ruma_api, receipt::ReceiptType, EventId, RoomId};

    ruma_api! {
        metadata: {
            description: "Send a receipt event to a room.",
            method: POST,
            name: "create_receipt",
            r0_path: "/_matrix/client/r0/rooms/:room_id/receipt/:receipt_type/:event_id",
            stable_path: "/_matrix/client/v3/rooms/:room_id/receipt/:receipt_type/:event_id",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
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
            Self {}
        }
    }
}
