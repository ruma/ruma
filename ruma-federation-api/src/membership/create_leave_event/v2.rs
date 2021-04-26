//! [PUT /_matrix/federation/v2/send_leave/{roomId}/{eventId}](https://matrix.org/docs/spec/server_server/r0.1.4#put-matrix-federation-v2-send-leave-roomid-eventid)

use ruma_api::ruma_api;
use ruma_events::pdu::Pdu;
use ruma_identifiers::{EventId, RoomId};
use ruma_serde::Raw;

ruma_api! {
    metadata: {
        description: "Submits a signed leave event to the receiving server for it to accept it into the room's graph.",
        name: "create_leave_event",
        method: PUT,
        path: "/_matrix/federation/v1/send_leave/:room_id/:event_id",
        rate_limited: false,
        authentication: ServerSignatures,
    }

    request: {
        /// The room ID that is about to be left.
        ///
        /// Do not use this. Instead, use the `room_id` field inside the PDU.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The event ID for the leave event.
        #[ruma_api(path)]
        pub event_id: &'a EventId,

        /// The PDU.
        #[ruma_api(body)]
        pub pdu: Raw<Pdu>,
    }

    #[derive(Default)]
    response: {}
}

impl<'a> Request<'a> {
    /// Creates a `Request` from the given room ID, event ID and `Pdu`.
    pub fn new(room_id: &'a RoomId, event_id: &'a EventId, pdu: Raw<Pdu>) -> Self {
        Self { room_id, event_id, pdu }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use ruma_api::OutgoingResponse;

    use super::Response;

    #[test]
    fn response_body() {
        let res = Response::new().try_into_http_response::<Vec<u8>>().unwrap();

        assert_eq!(res.body(), b"{}");
    }
}
