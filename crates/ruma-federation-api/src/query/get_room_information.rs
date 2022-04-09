//! `GET /_matrix/federation/*/query/directory`
//!
//! Endpoint to query room information with a room alias.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/server-server-api/#get_matrixfederationv1querydirectory

    use ruma_common::{api::ruma_api, OwnedRoomId, OwnedServerName, RoomAliasId};

    ruma_api! {
        metadata: {
            description: "Get mapped room ID and resident homeservers for a given room alias.",
            name: "get_room_information",
            method: GET,
            stable_path: "/_matrix/federation/v1/query/directory",
            rate_limited: false,
            authentication: ServerSignatures,
            added: 1.0,
        }

        request: {
            /// Room alias to query.
            #[ruma_api(query)]
            pub room_alias: &'a RoomAliasId,
        }

        response: {
            /// Room ID mapped to queried alias.
            pub room_id: OwnedRoomId,

            /// An array of server names that are likely to hold the given room.
            pub servers: Vec<OwnedServerName>,
        }
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room alias ID.
        pub fn new(room_alias: &'a RoomAliasId) -> Self {
            Self { room_alias }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room IDs and servers.
        pub fn new(room_id: OwnedRoomId, servers: Vec<OwnedServerName>) -> Self {
            Self { room_id, servers }
        }
    }
}
