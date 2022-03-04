//! `DELETE /_matrix/client/*/directory/room/{roomAlias}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#delete_matrixclientv3directoryroomroomalias

    use ruma_common::{api::ruma_api, RoomAliasId};

    ruma_api! {
        metadata: {
            description: "Remove an alias from a room.",
            method: DELETE,
            name: "delete_alias",
            r0_path: "/_matrix/client/r0/directory/room/:room_alias",
            stable_path: "/_matrix/client/v3/directory/room/:room_alias",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The room alias to remove.
            #[ruma_api(path)]
            pub room_alias: &'a RoomAliasId,
        }

        #[derive(Default)]
        response: {}

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room alias.
        pub fn new(room_alias: &'a RoomAliasId) -> Self {
            Self { room_alias }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
