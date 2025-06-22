//! `/v2/` ([spec])
//!
//! [spec]: https://spec.matrix.org/latest/server-server-api/#put_matrixfederationv2send_leaveroomideventid

use ruma_common::{
    api::{request, response, Metadata},
    metadata, OwnedEventId, OwnedRoomId,
};
use serde_json::value::RawValue as RawJsonValue;

const METADATA: Metadata = metadata! {
    method: PUT,
    rate_limited: false,
    authentication: ServerSignatures,
    history: {
        1.0 => "/_matrix/federation/v2/send_leave/{room_id}/{event_id}",
    }
};

/// Request type for the `create_leave_event` endpoint.
#[request]
pub struct Request {
    /// The room ID that is about to be left.
    ///
    /// Do not use this. Instead, use the `room_id` field inside the PDU.
    #[ruma_api(path)]
    pub room_id: OwnedRoomId,

    /// The event ID for the leave event.
    #[ruma_api(path)]
    pub event_id: OwnedEventId,

    /// The PDU.
    #[ruma_api(body)]
    pub pdu: Box<RawJsonValue>,
}

/// Response type for the `create_leave_event` endpoint.
#[response]
#[derive(Default)]
pub struct Response {}

impl Request {
    /// Creates a new `Request` from the given room ID, event ID and PDU.
    pub fn new(room_id: OwnedRoomId, event_id: OwnedEventId, pdu: Box<RawJsonValue>) -> Self {
        Self { room_id, event_id, pdu }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use ruma_common::api::OutgoingResponse;

    use super::Response;

    #[test]
    fn response_body() {
        let res = Response::new().try_into_http_response::<Vec<u8>>().unwrap();

        assert_eq!(res.body(), b"{}");
    }
}
