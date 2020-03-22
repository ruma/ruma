//! [PUT /_matrix/client/r0/user/{userId}/rooms/{roomId}/tags/{tag}](https://matrix.org/docs/spec/client_server/r0.4.0.html#put-matrix-client-r0-user-userid-rooms-roomid-tags-tag)

use ruma_api::ruma_api;
use ruma_events::tag::TagInfo;
use ruma_identifiers::{RoomId, UserId};

ruma_api! {
    metadata {
        description: "Add a new tag to a room.",
        method: PUT,
        name: "create_tag",
        path: "/_matrix/client/r0/user/:user_id/rooms/:room_id/tags/:tag",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The room to tag.
        #[ruma_api(path)]
        pub room_id: RoomId,
        /// The name of the tag to create.
        #[ruma_api(path)]
        pub tag: String,
        /// Info about the tag.
        #[ruma_api(body)]
        pub tag_info: TagInfo,
        /// The ID of the user creating the tag.
        #[ruma_api(path)]
        pub user_id: UserId,
    }

    response {}

    error: crate::Error
}
