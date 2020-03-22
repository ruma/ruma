//! [POST /_matrix/client/r0/rooms/{roomId}/report/{eventId}](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-rooms-roomid-report-eventid)

use js_int::Int;
use ruma_api::ruma_api;
use ruma_identifiers::{EventId, RoomId};

ruma_api! {
    metadata {
        description: "Report content as inappropriate.",
        method: POST,
        name: "report_content",
        path: "/rooms/:room_id/report/:event_id",
        rate_limited:  false,
        requires_authentication: true,
    }

    request {
        /// Room in which the event to be reported is located.
        #[ruma_api(path)]
        pub room_id: RoomId,
        /// Event to report.
        #[ruma_api(path)]
        pub event_id: EventId,
        /// Integer between -100 and 0 rating offensivness.
        pub score: Int,
        /// Reason to report content. May be blank.
        pub reason: String,
    }

    response {}

    error: crate::Error
}
