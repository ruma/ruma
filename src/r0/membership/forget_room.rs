//! [POST /_matrix/client/r0/rooms/{roomId}/forget](https://matrix.org/docs/spec/client_server/r0.4.0.html#post-matrix-client-r0-rooms-roomid-forget)

use ruma_api::ruma_api;
use ruma_identifiers::RoomId;

ruma_api! {
    metadata {
        description: "Forget a room.",
        method: POST,
        name: "forget_room",
        path: "/_matrix/client/r0/rooms/:room_id/forget",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// The room to forget.
        #[ruma_api(path)]
        pub room_id: RoomId,
    }

    response {}

    error: crate::Error
}
