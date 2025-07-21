//! `POST /_matrix/client/*/rooms/{roomId}/report/{eventId}`
//!
//! Report content as inappropriate.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3roomsroomidreporteventid

    use js_int::Int;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedEventId, OwnedRoomId,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/{room_id}/report/{event_id}",
            1.1 => "/_matrix/client/v3/rooms/{room_id}/report/{event_id}",
        }
    };

    /// Request type for the `report_content` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// Room in which the event to be reported is located.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// Event to report.
        #[ruma_api(path)]
        pub event_id: OwnedEventId,

        /// Integer between -100 and 0 rating offensivness.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub score: Option<Int>,

        /// Reason to report content.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<String>,
    }

    /// Response type for the `report_content` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given room ID, event ID, score and reason.
        pub fn new(
            room_id: OwnedRoomId,
            event_id: OwnedEventId,
            score: Option<Int>,
            reason: Option<String>,
        ) -> Self {
            Self { room_id, event_id, score, reason }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
