//! [PUT /_matrix/client/r0/directory/room/{roomAlias}](https://matrix.org/docs/spec/client_server/r0.6.0#put-matrix-client-r0-directory-room-roomalias)

use ruma_api::ruma_api;
use ruma_identifiers::{RoomAliasId, RoomId};

ruma_api! {
    metadata: {
        description: "Add an alias to a room.",
        method: PUT,
        name: "create_alias",
        path: "/_matrix/client/r0/directory/room/:room_alias",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The room alias to set.
        #[ruma_api(path)]
        pub room_alias: &'a RoomAliasId,

        /// The room ID to set.
        pub room_id: &'a RoomId,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room alias and room id.
    pub fn new(room_alias: &'a RoomAliasId, room_id: &'a RoomId) -> Self {
        Self { room_alias, room_id }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
