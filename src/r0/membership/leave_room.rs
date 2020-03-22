//! [POST /_matrix/client/r0/rooms/{roomId}/leave](https://matrix.org/docs/spec/client_server/r0.4.0.html#post-matrix-client-r0-rooms-roomid-leave)

use ruma_api::ruma_api;
use ruma_identifiers::RoomId;

ruma_api! {
    metadata {
        description: "Leave a room.",
        method: POST,
        name: "leave_room",
        path: "/_matrix/client/r0/rooms/:room_id/leave",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// The room to leave.
        #[ruma_api(path)]
        pub room_id: RoomId,
    }

    response {}

    error: crate::Error
}
