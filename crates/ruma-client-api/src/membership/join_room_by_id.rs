//! `POST /_matrix/client/*/rooms/{roomId}/join`
//!
//! Join a room using its ID.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3roomsroomidjoin

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId,
    };

    use crate::membership::ThirdPartySigned;

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/{room_id}/join",
            1.1 => "/_matrix/client/v3/rooms/{room_id}/join",
        }
    };

    /// Request type for the `join_room_by_id` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room where the user should be invited.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The signature of a `m.third_party_invite` token to prove that this user owns a third
        /// party identity which has been invited to the room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub third_party_signed: Option<ThirdPartySigned>,

        /// Optional reason for joining the room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<String>,
    }

    /// Response type for the `join_room_by_id` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The room that the user joined.
        pub room_id: OwnedRoomId,
    }

    impl Request {
        /// Creates a new `Request` with the given room id.
        pub fn new(room_id: OwnedRoomId) -> Self {
            Self { room_id, third_party_signed: None, reason: None }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room id.
        pub fn new(room_id: OwnedRoomId) -> Self {
            Self { room_id }
        }
    }
}
