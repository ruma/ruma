//! [GET /_matrix/client/r0/directory/room/{roomAlias}](https://matrix.org/docs/spec/client_server/r0.4.0.html#get-matrix-client-r0-directory-room-roomalias)

use ruma_api::ruma_api;
use ruma_identifiers::{RoomAliasId, RoomId};

ruma_api! {
    metadata {
        description: "Resolve a room alias to a room ID.",
        method: GET,
        name: "get_alias",
        path: "/_matrix/client/r0/directory/room/:room_alias",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The room alias.
        #[ruma_api(path)]
        pub room_alias: RoomAliasId,
    }

    response {
        /// The room ID for this room alias.
        pub room_id: RoomId,
        /// A list of servers that are aware of this room ID.
        pub servers: Vec<String>,
    }

    error: crate::Error
}
