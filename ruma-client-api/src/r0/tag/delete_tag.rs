//! [DELETE /_matrix/client/r0/user/{userId}/rooms/{roomId}/tags/{tag}](https://matrix.org/docs/spec/client_server/r0.6.0#delete-matrix-client-r0-user-userid-rooms-roomid-tags-tag)

use ruma_api::ruma_api;
use ruma_identifiers::{RoomId, UserId};

ruma_api! {
    metadata: {
        description: "Remove a tag from a room.",
        method: DELETE,
        name: "delete_tag",
        path: "/_matrix/client/r0/user/:user_id/rooms/:room_id/tags/:tag",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The user whose tag will be deleted.
        #[ruma_api(path)]
        pub user_id: &'a UserId,

        /// The tagged room.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The name of the tag to delete.
        #[ruma_api(path)]
        pub tag: &'a str,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given user ID, room ID and tag
    pub fn new(user_id: &'a UserId, room_id: &'a RoomId, tag: &'a str) -> Self {
        Self { user_id, room_id, tag }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
