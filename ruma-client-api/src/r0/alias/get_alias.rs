//! [GET /_matrix/client/r0/directory/room/{roomAlias}](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-directory-room-roomalias)

use ruma_api::ruma_api;
use ruma_identifiers::{RoomAliasId, RoomId};

ruma_api! {
    metadata: {
        description: "Resolve a room alias to a room ID.",
        method: GET,
        name: "get_alias",
        path: "/_matrix/client/r0/directory/room/:room_alias",
        rate_limited: false,
        requires_authentication: true,
    }

    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    request: {
        /// The room alias.
        #[ruma_api(path)]
        pub room_alias: &'a RoomAliasId,
    }

    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    response: {
        /// The room ID for this room alias.
        pub room_id: RoomId,

        /// A list of servers that are aware of this room ID.
        pub servers: Vec<String>,
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
    pub fn new(room_id: RoomId, servers: Vec<String>) -> Self {
        Self { room_id, servers }
    }
}
