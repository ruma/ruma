//! `POST /_matrix/client/*/rooms/{roomId}/kick`
//!
//! Kick a user from a room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3roomsroomidkick

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId, OwnedUserId,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/{room_id}/kick",
            1.1 => "/_matrix/client/v3/rooms/{room_id}/kick",
        }
    };

    /// Request type for the `kick_user` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room to kick the user from.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The user to kick.
        pub user_id: OwnedUserId,

        /// The reason for kicking the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<String>,
    }

    /// Response type for the `kick_user` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given room id and room id.
        pub fn new(room_id: OwnedRoomId, user_id: OwnedUserId) -> Self {
            Self { room_id, user_id, reason: None }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
