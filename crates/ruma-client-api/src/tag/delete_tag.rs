//! `DELETE /_matrix/client/*/user/{userId}/rooms/{roomId}/tags/{tag}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#put_matrixclientv3useruseridroomsroomidtagstag

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, RoomId, UserId,
    };

    const METADATA: Metadata = metadata! {
        description: "Remove a tag from a room.",
        method: DELETE,
        name: "delete_tag",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/user/:user_id/rooms/:room_id/tags/:tag",
            1.1 => "/_matrix/client/v3/user/:user_id/rooms/:room_id/tags/:tag",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The user whose tag will be deleted.
        #[ruma_api(path)]
        pub user_id: &'a UserId,

        /// The tagged room.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The name of the tag to delete.
        #[ruma_api(path)]
        pub tag: &'a str,
    }

    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given user ID, room ID and tag
        pub fn new(user_id: &'a UserId, room_id: &'a RoomId, tag: &'a str) -> Self {
            Self { user_id, room_id, tag }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
