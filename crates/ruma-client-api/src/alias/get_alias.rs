//! `GET /_matrix/client/*/directory/room/{roomAlias}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3directoryroomroomalias

    use ruma_common::{api::ruma_api, OwnedRoomId, OwnedServerName, RoomAliasId};

    ruma_api! {
        metadata: {
            description: "Resolve a room alias to a room ID.",
            method: GET,
            name: "get_alias",
            r0_path: "/_matrix/client/r0/directory/room/:room_alias",
            stable_path: "/_matrix/client/v3/directory/room/:room_alias",
            rate_limited: false,
            authentication: None,
            added: 1.0,
        }

        request: {
            /// The room alias.
            #[ruma_api(path)]
            pub room_alias: &'a RoomAliasId,
        }

        response: {
            /// The room ID for this room alias.
            pub room_id: OwnedRoomId,

            /// A list of servers that are aware of this room ID.
            pub servers: Vec<OwnedServerName>,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room alias id.
        pub fn new(room_alias: &'a RoomAliasId) -> Self {
            Self { room_alias }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room id and servers
        pub fn new(room_id: OwnedRoomId, servers: Vec<OwnedServerName>) -> Self {
            Self { room_id, servers }
        }
    }
}
