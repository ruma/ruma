//! `POST /_matrix/client/*/rooms/{roomId}/report/{eventId}`
//!
//! Report content as inappropriate.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#post_matrixclientv3roomsroomidreporteventid

    use js_int::Int;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, EventId, RoomId,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/:room_id/report/:event_id",
            1.1 => "/_matrix/client/v3/rooms/:room_id/report/:event_id",
        }
    };

    /// Request type for the `report_content` endpoint.
    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// Room in which the event to be reported is located.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// Event to report.
        #[ruma_api(path)]
        pub event_id: &'a EventId,

        /// Integer between -100 and 0 rating offensivness.
        pub score: Option<Int>,

        /// Reason to report content.
        ///
        /// May be blank.
        pub reason: Option<&'a str>,
    }

    /// Response type for the `report_content` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room ID, event ID, score and reason.
        pub fn new(
            room_id: &'a RoomId,
            event_id: &'a EventId,
            score: Option<Int>,
            reason: Option<&'a str>,
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
