//! [POST /_matrix/client/r0/rooms/{roomId}/kick](https://matrix.org/docs/spec/client_server/r0.4.0.html#post-matrix-client-r0-rooms-roomid-kick)

use ruma_api::ruma_api;
use ruma_identifiers::{RoomId, UserId};

ruma_api! {
    metadata {
        description: "Kick a user from a room.",
        method: POST,
        name: "kick_user",
        path: "/_matrix/client/r0/rooms/:room_id/kick",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The reason for kicking the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<String>,
        /// The room to kick the user from.
        #[ruma_api(path)]
        pub room_id: RoomId,
        /// The user to kick.
        pub user_id: UserId,
    }

    response {}

    error: crate::Error
}
