//! [GET /_matrix/federation/v1/query/directory](https://matrix.org/docs/spec/server_server/r0.1.3#get-matrix-federation-v1-query-directory)

use ruma_api::ruma_api;
use ruma_identifiers::RoomId;

ruma_api! {
    metadata: {
        description: "Get mapped room ID and resident homeservers for a given room alias.",
        name: "get_room_information",
        method: GET,
        path: "/_matrix/federation/v1/query/directory",
        rate_limited: false,
        requires_authentication: true,
    }

    request: {
        /// Room alias to query.
        #[ruma_api(query)]
        pub room_alias: String,
    }

    response: {
        /// Room ID mapped to queried alias.
        pub room_id: RoomId,
        /// An array of server names that are likely to hold the given room.
        pub servers: Vec<String>,
    }
}
