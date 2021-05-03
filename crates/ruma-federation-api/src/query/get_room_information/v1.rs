//! [GET /_matrix/federation/v1/query/directory](https://matrix.org/docs/spec/server_server/r0.1.3#get-matrix-federation-v1-query-directory)

use ruma_api::ruma_api;
use ruma_identifiers::{RoomAliasId, RoomId, ServerNameBox};

ruma_api! {
    metadata: {
        description: "Get mapped room ID and resident homeservers for a given room alias.",
        name: "get_room_information",
        method: GET,
        path: "/_matrix/federation/v1/query/directory",
        rate_limited: false,
        authentication: ServerSignatures,
    }

    request: {
        /// Room alias to query.
        #[ruma_api(query)]
        pub room_alias: &'a RoomAliasId,
    }

    response: {
        /// Room ID mapped to queried alias.
        pub room_id: RoomId,

        /// An array of server names that are likely to hold the given room.
        pub servers: Vec<ServerNameBox>,
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
    pub fn new(room_id: RoomId, servers: Vec<ServerNameBox>) -> Self {
        Self { room_id, servers }
    }
}
