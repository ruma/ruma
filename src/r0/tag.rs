//! Endpoints for tagging rooms.

/// [PUT /_matrix/client/r0/user/{userId}/rooms/{roomId}/tags/{tag}](https://matrix.org/docs/spec/client_server/r0.2.0.html#put-matrix-client-r0-user-userid-rooms-roomid-tags-tag)
pub mod create_tag {
    use ruma_api_macros::ruma_api;
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
    }
}

/// [GET /_matrix/client/r0/user/{userId}/rooms/{roomId}/tags](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-user-userid-rooms-roomid-tags)
pub mod get_tags {
    use ruma_api_macros::ruma_api;
    use ruma_events::tag::TagEventContent;
    use ruma_identifiers::{RoomId, UserId};

    ruma_api! {
        metadata {
            description: "Get the tags associated with a room.",
            method: GET,
            name: "get_tags",
            path: "/_matrix/client/r0/user/:user_id/rooms/:room_id/tags",
            rate_limited: false,
            requires_authentication: true,
        }

        request {
            /// The room from which tags will be retrieved.
            #[ruma_api(path)]
            pub room_id: RoomId,
            /// The user whose tags will be retrieved.
            #[ruma_api(path)]
            pub user_id: UserId,
        }

        response {
            /// The user's tags for the room.
            pub tags: TagEventContent,
        }
    }
}

/// [DELETE /_matrix/client/r0/user/{userId}/rooms/{roomId}/tags/{tag}](https://matrix.org/docs/spec/client_server/r0.2.0.html#delete-matrix-client-r0-user-userid-rooms-roomid-tags-tag)
pub mod delete_tag {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::{RoomId, UserId};

    ruma_api! {
        metadata {
            description: "Remove a tag from a room.",
            method: DELETE,
            name: "delete_tag",
            path: "/_matrix/client/r0/user/:user_id/rooms/:room_id/tags/:tag",
            rate_limited: false,
            requires_authentication: true,
        }

        request {
            /// The tagged room.
            #[ruma_api(path)]
            pub room_id: RoomId,
            /// The name of the tag to delete.
            #[ruma_api(path)]
            pub tag: String,
            /// The user whose tag will be deleted.
            #[ruma_api(path)]
            pub user_id: UserId,
        }

        response {}
    }
}
