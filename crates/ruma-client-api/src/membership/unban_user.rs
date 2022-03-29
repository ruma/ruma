//! `POST /_matrix/client/*/rooms/{roomId}/unban`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3roomsroomidunban

    use ruma_common::{api::ruma_api, RoomId, UserId};

    ruma_api! {
        metadata: {
            description: "Unban a user from a room.",
            method: POST,
            name: "unban_user",
            r0_path: "/_matrix/client/r0/rooms/:room_id/unban",
            stable_path: "/_matrix/client/v3/rooms/:room_id/unban",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The room to unban the user from.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,

            /// The user to unban.
            pub user_id: &'a UserId,

            /// Optional reason for unbanning the user.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub reason: Option<&'a str>,
        }

        #[derive(Default)]
        response: {}

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room id and room id.
        pub fn new(room_id: &'a RoomId, user_id: &'a UserId) -> Self {
            Self { room_id, user_id, reason: None }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
