//! [POST /_matrix/client/r0/rooms/{roomId}/invite](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-rooms-roomid-invite)

use ruma_api_macros::ruma_api;
use ruma_identifiers::{RoomId, UserId};
use serde_derive::{Deserialize, Serialize};

ruma_api! {
    metadata {
        description: "Invite a user to a room.",
        method: POST,
        name: "invite_user",
        path: "/_matrix/client/r0/rooms/:room_id/invite",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// The room where the user should be invited.
        #[ruma_api(path)]
        pub room_id: RoomId,
        /// The user to invite.
        pub user_id: UserId,
    }

    response {}
}
