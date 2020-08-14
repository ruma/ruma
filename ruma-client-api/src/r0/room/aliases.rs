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
        requires_authentication: true,
    }

    request: {
        /// The room ID to get aliases of.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,
    }

    response: {
        aliases: Vec<RoomAliasId>,
    }

    error: crate::Error
}
