//! [POST /_matrix/client/r0/rooms/{roomId}/kick](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-rooms-roomid-kick)

use ruma_api::ruma_api;
use ruma_identifiers::{RoomId, UserId};

ruma_api! {
    metadata: {
        description: "Kick a user from a room.",
        method: POST,
        name: "kick_user",
        path: "/_matrix/client/r0/rooms/:room_id/kick",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The room to kick the user from.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The user to kick.
        pub user_id: &'a UserId,

        /// The reason for kicking the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<&'a str>,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room id and room id.
    pub fn new(room_id: &'a RoomId, user_id: &'a UserId) -> Self {
        Self { room_id, user_id, reason: None }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
