//! `POST /_matrix/client/*/rooms/{roomId}/read_markers`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! This endpoint is equivalent to calling the [`create_receipt`] endpoint,
    //! but is provided as a way to update several read markers with a single call.
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3roomsroomidread_markers
    //! [`create_receipt`]: crate::receipt::create_receipt

    use ruma_common::{api::ruma_api, EventId, RoomId};

    ruma_api! {
        metadata: {
            description: "Sets the position of the read marker for a given room, and optionally the read receipt's location.",
            method: POST,
            name: "set_read_marker",
            r0_path: "/_matrix/client/r0/rooms/:room_id/read_markers",
            stable_path: "/_matrix/client/v3/rooms/:room_id/read_markers",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The room ID to set the read marker in for the user.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,

            /// The event ID the fully-read marker should be located at.
            ///
            /// The event MUST belong to the room.
            ///
            /// This is equivalent to calling the [`create_receipt`] endpoint with a
            /// [`ReceiptType::FullyRead`].
            ///
            /// [`create_receipt`]: crate::receipt::create_receipt
            /// [`ReceiptType::FullyRead`]: crate::receipt::create_receipt::v3::ReceiptType::FullyRead
            #[serde(rename = "m.fully_read", skip_serializing_if = "Option::is_none")]
            pub fully_read: Option<&'a EventId>,

            /// The event ID to set the public read receipt location at.
            ///
            /// This is equivalent to calling the [`create_receipt`] endpoint with a
            /// [`ReceiptType::Read`].
            ///
            /// [`create_receipt`]: crate::receipt::create_receipt
            /// [`ReceiptType::Read`]: crate::receipt::create_receipt::v3::ReceiptType::Read
            #[serde(rename = "m.read", skip_serializing_if = "Option::is_none")]
            pub read_receipt: Option<&'a EventId>,

            /// The event ID to set the private read receipt location at.
            ///
            /// This is equivalent to calling the [`create_receipt`] endpoint with a
            /// [`ReceiptType::ReadPrivate`].
            ///
            /// [`create_receipt`]: crate::receipt::create_receipt
            /// [`ReceiptType::ReadPrivate`]: crate::receipt::create_receipt::v3::ReceiptType::ReadPrivate
            #[serde(rename = "m.read.private", skip_serializing_if = "Option::is_none")]
            pub private_read_receipt: Option<&'a EventId>,
        }

        #[derive(Default)]
        response: {}

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room ID.
        pub fn new(room_id: &'a RoomId) -> Self {
            Self { room_id, fully_read: None, read_receipt: None, private_read_receipt: None }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
