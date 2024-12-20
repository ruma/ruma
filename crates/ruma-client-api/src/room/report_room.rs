//! `POST /_matrix/client/*/rooms/{roomId}/report`
//!
//! Report a room as inappropriate.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3roomsroomidreport

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc4151/rooms/:room_id/report",
            1.13 => "/_matrix/client/v3/rooms/:room_id/report",
        }
    };

    /// Request type for the `report_room` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The ID of the room to report.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The reason to report the room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<String>,
    }

    /// Response type for the `report_room` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given room ID.
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
