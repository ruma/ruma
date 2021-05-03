//! [GET /_matrix/client/r0/rooms/{roomId}/aliases](https://matrix.org/docs/spec/client_server/r0.6.1#get-matrix-client-r0-rooms-roomid-aliases)

use ruma_api::ruma_api;
use ruma_identifiers::{RoomAliasId, RoomId};

ruma_api! {
    metadata: {
        description: "Get a list of aliases maintained by the local server for the given room.",
        method: GET,
        name: "aliases",
        path: "/_matrix/client/r0/rooms/:room_id/aliases",
        rate_limited: true,
        authentication: AccessToken,
    }

    request: {
        /// The room ID to get aliases of.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,
    }

    response: {
        /// The server's local aliases on the room.
        pub aliases: Vec<RoomAliasId>,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room ID.
    pub fn new(room_id: &'a RoomId) -> Self {
        Self { room_id }
    }
}

impl Response {
    /// Creates a new `Response` with the given aliases.
    pub fn new(aliases: Vec<RoomAliasId>) -> Self {
        Self { aliases }
    }
}
