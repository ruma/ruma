//! `DELETE /_matrix/client/*/user/{userId}/rooms/{roomId}/tags/{tag}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#put_matrixclientv3useruseridroomsroomidtagstag

    use ruma_common::{api::ruma_api, RoomId, UserId};

    ruma_api! {
        metadata: {
            description: "Remove a tag from a room.",
            method: DELETE,
            name: "delete_tag",
            r0_path: "/_matrix/client/r0/user/:user_id/rooms/:room_id/tags/:tag",
            stable_path: "/_matrix/client/v3/user/:user_id/rooms/:room_id/tags/:tag",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The user whose tag will be deleted.
            #[ruma_api(path)]
            pub user_id: &'a UserId,

            /// The tagged room.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,

            /// The name of the tag to delete.
            #[ruma_api(path)]
            pub tag: &'a str,
        }

        #[derive(Default)]
        response: {}

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given user ID, room ID and tag
        pub fn new(user_id: &'a UserId, room_id: &'a RoomId, tag: &'a str) -> Self {
            Self { user_id, room_id, tag }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
