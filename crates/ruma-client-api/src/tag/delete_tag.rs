//! `DELETE /_matrix/client/*/user/{userId}/rooms/{roomId}/tags/{tag}`
//!
//! Remove a tag from a room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixclientv3useruseridroomsroomidtagstag

    use ruma_common::{
        RoomId, UserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: DELETE,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/user/{user_id}/rooms/{room_id}/tags/{tag}",
            1.1 => "/_matrix/client/v3/user/{user_id}/rooms/{room_id}/tags/{tag}",
        }
    }

    /// Request type for the `delete_tag` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The user whose tag will be deleted.
        #[ruma_api(path)]
        pub user_id: UserId,

        /// The tagged room.
        #[ruma_api(path)]
        pub room_id: RoomId,

        /// The name of the tag to delete.
        #[ruma_api(path)]
        pub tag: String,
    }

    /// Response type for the `delete_tag` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given user ID, room ID and tag
        pub fn new(user_id: UserId, room_id: RoomId, tag: String) -> Self {
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
