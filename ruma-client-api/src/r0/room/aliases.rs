//! [PUT /_matrix/client/r0/directory/room/{roomAlias}](https://matrix.org/docs/spec/client_server/r0.6.0#put-matrix-client-r0-directory-room-roomalias)

use ruma_api::ruma_api;
use ruma_identifiers::{RoomAliasId, RoomId};

ruma_api! {
    metadata: {
        description: "Get a list of local aliases on a given room.",
        method: PUT,
        name: "create_alias",
        path: "/_matrix/client/r0/directory/room/:room_id",
        rate_limited: false,
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
