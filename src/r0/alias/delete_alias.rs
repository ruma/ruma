//! [DELETE /_matrix/client/r0/directory/room/{roomAlias}](https://matrix.org/docs/spec/client_server/r0.4.0.html#delete-matrix-client-r0-directory-room-roomalias)

use ruma_api::ruma_api;
use ruma_identifiers::RoomAliasId;

ruma_api! {
    metadata {
        description: "Remove an alias from a room.",
        method: DELETE,
        name: "delete_alias",
        path: "/_matrix/client/r0/directory/room/:room_alias",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The room alias to remove.
        #[ruma_api(path)]
        pub room_alias: RoomAliasId,
    }

    response {}

    error: crate::Error
}
