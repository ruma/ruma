//! [POST /_matrix/client/r0/rooms/{roomId}/join](https://matrix.org/docs/spec/client_server/r0.4.0.html#post-matrix-client-r0-rooms-roomid-join)

use ruma_api::ruma_api;
use ruma_identifiers::RoomId;

use super::ThirdPartySigned;

ruma_api! {
    metadata {
        description: "Join a room using its ID.",
        method: POST,
        name: "join_room_by_id",
        path: "/_matrix/client/r0/rooms/:room_id/join",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// The room where the user should be invited.
        #[ruma_api(path)]
        pub room_id: RoomId,
        /// The signature of a `m.third_party_invite` token to prove that this user owns a third
        /// party identity which has been invited to the room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub third_party_signed: Option<ThirdPartySigned>,
    }

    response {
        /// The room that the user joined.
        pub room_id: RoomId,
    }

    error: crate::Error
}
