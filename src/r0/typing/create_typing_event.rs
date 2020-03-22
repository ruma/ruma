//! [PUT /_matrix/client/r0/rooms/{roomId}/typing/{userId}](https://matrix.org/docs/spec/client_server/r0.4.0.html#put-matrix-client-r0-rooms-roomid-typing-userid)

use std::time::Duration;

use ruma_api::ruma_api;
use ruma_identifiers::{RoomId, UserId};

ruma_api! {
    metadata {
        method: PUT,
        path: "/_matrix/client/r0/rooms/:room_id/typing/:user_id",
        name: "create_typing_event",
        description: "Send a typing event to a room.",
        requires_authentication: true,
        rate_limited: true,
    }

    request {
        /// The room in which the user is typing.
        #[ruma_api(path)]
        pub room_id: RoomId,
        /// The length of time in milliseconds to mark this user as typing.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default, with = "crate::serde::duration::opt_ms")]
        pub timeout: Option<Duration>,
        /// Whether the user is typing or not. If `false`, the `timeout` key can be omitted.
        pub typing: bool,
        /// The user who has started to type.
        #[ruma_api(path)]
        pub user_id: UserId,
    }

    response {}

    error: crate::Error
}
