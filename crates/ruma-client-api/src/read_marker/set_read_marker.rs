//! `POST /_matrix/client/*/rooms/{roomId}/read_markers`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3roomsroomidread_markers

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

            /// The event ID the read marker should be located at.
            ///
            /// The event MUST belong to the room.
            #[serde(rename = "m.fully_read")]
            pub fully_read: &'a EventId,

            /// The event ID to set the read receipt location at.
            ///
            /// This is equivalent to calling the create_read_receipt endpoint and is provided here to
            /// save that extra call.
            #[serde(rename = "m.read", skip_serializing_if = "Option::is_none")]
            pub read_receipt: Option<&'a EventId>,
        }

        #[derive(Default)]
        response: {}

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room ID and fully read event ID.
        pub fn new(room_id: &'a RoomId, fully_read: &'a EventId) -> Self {
            Self { room_id, fully_read, read_receipt: None }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
