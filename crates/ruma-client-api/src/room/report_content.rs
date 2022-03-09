//! `POST /_matrix/client/*/rooms/{roomId}/report/{eventId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3roomsroomidreporteventid

    use js_int::Int;
    use ruma_common::{api::ruma_api, EventId, RoomId};

    ruma_api! {
        metadata: {
            description: "Report content as inappropriate.",
            method: POST,
            name: "report_content",
            r0_path: "/_matrix/client/r0/rooms/:room_id/report/:event_id",
            stable_path: "/_matrix/client/v3/rooms/:room_id/report/:event_id",
            rate_limited:  false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
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

        #[derive(Default)]
        response: {}

        error: crate::Error
    }

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
