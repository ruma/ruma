//! `PUT /_matrix/client/*/user/{userId}/rooms/{roomId}/tags/{tag}`
//!
//! Add a new tag to a room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixclientv3useruseridroomsroomidtagstag

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId, OwnedUserId,
    };
    use ruma_events::tag::TagInfo;

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/user/{user_id}/rooms/{room_id}/tags/{tag}",
            1.1 => "/_matrix/client/v3/user/{user_id}/rooms/{room_id}/tags/{tag}",
        }
    };

    /// Request type for the `create_tag` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The ID of the user creating the tag.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        /// The room to tag.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The name of the tag to create.
        #[ruma_api(path)]
        pub tag: String,

        /// Info about the tag.
        #[ruma_api(body)]
        pub tag_info: TagInfo,
    }

    /// Response type for the `create_tag` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given user ID, room ID, tag and tag info.
        pub fn new(
            user_id: OwnedUserId,
            room_id: OwnedRoomId,
            tag: String,
            tag_info: TagInfo,
        ) -> Self {
            Self { user_id, room_id, tag, tag_info }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
