//! [POST /_matrix/client/r0/rooms/{roomId}/unban](https://matrix.org/docs/spec/client_server/r0.4.0.html#post-matrix-client-r0-rooms-roomid-unban)

use ruma_api::ruma_api;
use ruma_identifiers::{RoomId, UserId};

ruma_api! {
    metadata {
        description: "Unban a user from a room.",
        method: POST,
        name: "unban_user",
        path: "/_matrix/client/r0/rooms/:room_id/unban",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The room to unban the user from.
        #[ruma_api(path)]
        pub room_id: RoomId,
        /// The user to unban.
        pub user_id: UserId,
    }

    response {}

    error: crate::Error
}
