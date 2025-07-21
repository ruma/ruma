//! `POST /_matrix/client/*/rooms/{roomId}/receipt/{receiptType}/{eventId}`
//!
//! Send a receipt event to a room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3roomsroomidreceiptreceipttypeeventid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::{OrdAsRefStr, PartialEqAsRefStr, PartialOrdAsRefStr, StringEnum},
        OwnedEventId, OwnedRoomId,
    };
    use ruma_events::receipt::ReceiptThread;

    use crate::PrivOwnedStr;

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/{room_id}/receipt/{receipt_type}/{event_id}",
            1.1 => "/_matrix/client/v3/rooms/{room_id}/receipt/{receipt_type}/{event_id}",
        }
    };

    /// Request type for the `create_receipt` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room in which to send the event.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The type of receipt to send.
        #[ruma_api(path)]
        pub receipt_type: ReceiptType,

        /// The event ID to acknowledge up to.
        #[ruma_api(path)]
        pub event_id: OwnedEventId,

        /// The thread this receipt applies to.
        ///
        /// *Note* that this must be the default value if used with
        /// [`ReceiptType::FullyRead`].
        ///
        /// Defaults to [`ReceiptThread::Unthreaded`].
        #[serde(
            rename = "thread_id",
            default,
            skip_serializing_if = "ruma_common::serde::is_default"
        )]
        pub thread: ReceiptThread,
    }

    /// Response type for the `create_receipt` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given room ID, receipt type and event ID.
        pub fn new(
            room_id: OwnedRoomId,
            receipt_type: ReceiptType,
            event_id: OwnedEventId,
        ) -> Self {
            Self { room_id, receipt_type, event_id, thread: ReceiptThread::default() }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }

    /// The type of receipt.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
    #[derive(Clone, PartialOrdAsRefStr, OrdAsRefStr, PartialEqAsRefStr, Eq, StringEnum)]
    #[non_exhaustive]
    pub enum ReceiptType {
        /// A [public read receipt].
        ///
        /// Indicates that the given event has been presented to the user.
        ///
        /// This receipt is federated to other users.
        ///
        /// [public read receipt]: https://spec.matrix.org/latest/client-server-api/#receipts
        #[ruma_enum(rename = "m.read")]
        Read,

        /// A [private read receipt].
        ///
        /// Indicates that the given event has been presented to the user.
        ///
        /// This read receipt is not federated so only the user and their homeserver
        /// are aware of it.
        ///
        /// [private read receipt]: https://spec.matrix.org/latest/client-server-api/#private-read-receipts
        #[ruma_enum(rename = "m.read.private")]
        ReadPrivate,

        /// A [fully read marker].
        ///
        /// Indicates that the given event has been read by the user.
        ///
        /// This is actually not a receipt, but a piece of room account data. It is
        /// provided here for convenience.
        ///
        /// [fully read marker]: https://spec.matrix.org/latest/client-server-api/#fully-read-markers
        #[ruma_enum(rename = "m.fully_read")]
        FullyRead,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }
}
