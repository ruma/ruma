//! [POST /_matrix/client/r0/knock/{roomIdOrAlias}](https://spec.matrix.org/unstable/client-server-api/#post_matrixclientr0knockroomidoralias)

use ruma_api::ruma_api;
use ruma_identifiers::{RoomId, RoomOrAliasId, ServerName};

ruma_api! {
    metadata: {
        description: "Knock on a room.",
        method: POST,
        name: "knock_room",
        unstable_path: "/_matrix/client/unstable/xyz.amorgan.knock/knock/:room_id_or_alias",
        stable_path: "/_matrix/client/v3/knock/:room_id_or_alias",
        rate_limited: true,
        authentication: AccessToken,
        added: 1.1,
    }

    request: {
        /// The room the user should knock on.
        #[ruma_api(path)]
        pub room_id_or_alias: &'a RoomOrAliasId,

        /// The reason for joining a room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<&'a str>,

        /// The servers to attempt to knock on the room through.
        ///
        /// One of the servers must be participating in the room.
        #[ruma_api(query)]
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub server_name: &'a [Box<ServerName>],
    }

    response: {
        /// The room that the user knocked on.
        pub room_id: Box<RoomId>,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room ID or alias.
    pub fn new(room_id_or_alias: &'a RoomOrAliasId) -> Self {
        Self { room_id_or_alias, reason: None, server_name: &[] }
    }
}

impl Response {
    /// Creates a new `Response` with the given room ID.
    pub fn new(room_id: Box<RoomId>) -> Self {
        Self { room_id }
    }
}
