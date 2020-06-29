//! [POST /_matrix/client/r0/rooms/{roomId}/ban](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-rooms-roomid-ban)

use ruma_api::ruma_api;
use ruma_identifiers::{RoomId, UserId};

ruma_api! {
    metadata: {
        description: "Ban a user from a room.",
        method: POST,
        name: "ban_user",
        path: "/_matrix/client/r0/rooms/:room_id/ban",
        rate_limited: false,
        requires_authentication: true,
    }

    request: {
        /// The room to kick the user from.
        #[ruma_api(path)]
        pub room_id: RoomId,

        /// The user to ban.
        pub user_id: UserId,

        /// The reason for banning the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<String>,
    }

    response: {}

    error: crate::Error
}
