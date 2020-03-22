//! [PUT /_matrix/client/r0/directory/room/{roomAlias}](https://matrix.org/docs/spec/client_server/r0.6.0#put-matrix-client-r0-directory-room-roomalias)

use ruma_api::ruma_api;
use ruma_identifiers::{RoomAliasId, RoomId};

ruma_api! {
    metadata {
        description: "Add an alias to a room.",
        method: PUT,
        name: "create_alias",
        path: "/_matrix/client/r0/directory/room/:room_alias",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The room alias to set.
        #[ruma_api(path)]
        pub room_alias: RoomAliasId,
        /// The room ID to set.
        pub room_id: RoomId,
    }

    response {}

    error: crate::Error
}
