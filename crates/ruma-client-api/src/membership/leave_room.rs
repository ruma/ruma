//! `POST /_matrix/client/*/rooms/{roomId}/leave`
//!
//! Leave a room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3roomsroomidleave

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/{room_id}/leave",
            1.1 => "/_matrix/client/v3/rooms/{room_id}/leave",
        }
    };

    /// Request type for the `leave_room` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room to leave.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// Optional reason to be included as the `reason` on the subsequent membership event.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<String>,
    }

    /// Response type for the `leave_room` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given room id.
        pub fn new(room_id: OwnedRoomId) -> Self {
            Self { room_id, reason: None }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
