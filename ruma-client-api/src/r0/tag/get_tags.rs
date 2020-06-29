//! [GET /_matrix/client/r0/user/{userId}/rooms/{roomId}/tags](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-user-userid-rooms-roomid-tags)

use ruma_api::ruma_api;
use ruma_events::{tag::TagEventContent, EventJson};
use ruma_identifiers::{RoomId, UserId};

ruma_api! {
    metadata: {
        description: "Get the tags associated with a room.",
        method: GET,
        name: "get_tags",
        path: "/_matrix/client/r0/user/:user_id/rooms/:room_id/tags",
        rate_limited: false,
        requires_authentication: true,
    }

    request: {
        /// The user whose tags will be retrieved.
        #[ruma_api(path)]
        pub user_id: UserId,

        /// The room from which tags will be retrieved.
        #[ruma_api(path)]
        pub room_id: RoomId,
    }

    response: {
        /// The user's tags for the room.
        pub tags: EventJson<TagEventContent>,
    }

    error: crate::Error
}
