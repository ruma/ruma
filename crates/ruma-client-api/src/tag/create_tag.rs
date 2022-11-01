//! `PUT /_matrix/client/*/user/{userId}/rooms/{roomId}/tags/{tag}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#put_matrixclientv3useruseridroomsroomidtagstag

    use ruma_common::{
        api::{request, response, Metadata},
        events::tag::TagInfo,
        metadata, RoomId, UserId,
    };

    const METADATA: Metadata = metadata! {
        description: "Add a new tag to a room.",
        method: PUT,
        name: "create_tag",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/user/:user_id/rooms/:room_id/tags/:tag",
            1.1 => "/_matrix/client/v3/user/:user_id/rooms/:room_id/tags/:tag",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The ID of the user creating the tag.
        #[ruma_api(path)]
        pub user_id: &'a UserId,

        /// The room to tag.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The name of the tag to create.
        #[ruma_api(path)]
        pub tag: &'a str,

        /// Info about the tag.
        #[ruma_api(body)]
        pub tag_info: TagInfo,
    }

    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given user ID, room ID, tag and tag info.
        pub fn new(
            user_id: &'a UserId,
            room_id: &'a RoomId,
            tag: &'a str,
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
