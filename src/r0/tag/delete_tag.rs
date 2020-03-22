//! [DELETE /_matrix/client/r0/user/{userId}/rooms/{roomId}/tags/{tag}](https://matrix.org/docs/spec/client_server/r0.4.0.html#delete-matrix-client-r0-user-userid-rooms-roomid-tags-tag)

use ruma_api::ruma_api;
use ruma_identifiers::{RoomId, UserId};

ruma_api! {
    metadata {
        description: "Remove a tag from a room.",
        method: DELETE,
        name: "delete_tag",
        path: "/_matrix/client/r0/user/:user_id/rooms/:room_id/tags/:tag",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The tagged room.
        #[ruma_api(path)]
        pub room_id: RoomId,
        /// The name of the tag to delete.
        #[ruma_api(path)]
        pub tag: String,
        /// The user whose tag will be deleted.
        #[ruma_api(path)]
        pub user_id: UserId,
    }

    response {}

    error: crate::Error
}
