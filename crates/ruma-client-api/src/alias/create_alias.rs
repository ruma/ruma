//! `PUT /_matrix/client/*/directory/room/{roomAlias}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#put_matrixclientv3directoryroomroomalias

    use ruma_common::{api::ruma_api, RoomAliasId, RoomId};

    ruma_api! {
        metadata: {
            description: "Add an alias to a room.",
            method: PUT,
            name: "create_alias",
            r0_path: "/_matrix/client/r0/directory/room/:room_alias",
            stable_path: "/_matrix/client/v3/directory/room/:room_alias",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The room alias to set.
            #[ruma_api(path)]
            pub room_alias: &'a RoomAliasId,

            /// The room ID to set.
            pub room_id: &'a RoomId,
        }

        #[derive(Default)]
        response: {}

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room alias and room id.
        pub fn new(room_alias: &'a RoomAliasId, room_id: &'a RoomId) -> Self {
            Self { room_alias, room_id }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
