//! `POST /_matrix/client/*/rooms/{roomId}/report`
//!
//! Report a room as inappropriate.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3roomsroomidreport

    use ruma_common::{
        RoomId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc4151/rooms/{room_id}/report",
            1.13 => "/_matrix/client/v3/rooms/{room_id}/report",
        }
    }

    /// Request type for the `report_room` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The ID of the room to report.
        #[ruma_api(path)]
        pub room_id: RoomId,

        /// The reason to report the room, may be empty.
        // We deserialize a missing field as an empty string for backwards compatibility. The field
        // was initially specified as optional in Matrix 1.13 and then clarified as being required
        // in Matrix 1.14 to align with its initial definition in MSC4151.
        // See: https://github.com/matrix-org/matrix-spec/pull/2093#discussion_r1993171166
        #[serde(default)]
        pub reason: String,
    }

    /// Response type for the `report_room` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given room ID and reason.
        pub fn new(room_id: RoomId, reason: String) -> Self {
            Self { room_id, reason }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
