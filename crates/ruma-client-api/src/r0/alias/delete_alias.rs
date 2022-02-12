//! [DELETE /_matrix/client/r0/directory/room/{roomAlias}](https://matrix.org/docs/spec/client_server/r0.6.1#delete-matrix-client-r0-directory-room-roomalias)

use ruma_api::ruma_api;
use ruma_identifiers::RoomAliasId;

ruma_api! {
    metadata: {
        description: "Remove an alias from a room.",
        method: DELETE,
        name: "delete_alias",
        r0_path: "/_matrix/client/r0/directory/room/:room_alias",
        stable_path: "/_matrix/client/v3/directory/room/:room_alias",
        rate_limited: false,
        authentication: AccessToken,
        added: 1.0,
    }

    request: {
        /// The room alias to remove.
        #[ruma_api(path)]
        pub room_alias: &'a RoomAliasId,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room alias.
    pub fn new(room_alias: &'a RoomAliasId) -> Self {
        Self { room_alias }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self {}
    }
}
