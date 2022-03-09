//! `POST /_matrix/client/*/rooms/{roomId}/forget`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3roomsroomidforget

    use ruma_common::{api::ruma_api, RoomId};

    ruma_api! {
        metadata: {
            description: "Forget a room.",
            method: POST,
            name: "forget_room",
            r0_path: "/_matrix/client/r0/rooms/:room_id/forget",
            stable_path: "/_matrix/client/v3/rooms/:room_id/forget",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The room to forget.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,
        }

        #[derive(Default)]
        response: {}

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room id.
        pub fn new(room_id: &'a RoomId) -> Self {
            Self { room_id }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
