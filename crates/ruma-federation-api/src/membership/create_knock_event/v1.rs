//! `/v1/` ([spec])
//!
//! [spec]: https://spec.matrix.org/latest/server-server-api/#put_matrixfederationv1send_knockroomideventid

use ruma_common::{
    EventId, RoomId,
    api::{request, response},
    metadata,
};
use serde_json::value::RawValue as RawJsonValue;

use crate::{authentication::ServerSignatures, membership::RawStrippedState};

metadata! {
    method: PUT,
    rate_limited: false,
    authentication: ServerSignatures,
    path: "/_matrix/federation/v1/send_knock/{room_id}/{event_id}",
}

/// Request type for the `send_knock` endpoint.
#[request]
pub struct Request {
    /// The room ID that should receive the knock.
    #[ruma_api(path)]
    pub room_id: RoomId,

    /// The event ID for the knock event.
    #[ruma_api(path)]
    pub event_id: EventId,

    /// The PDU.
    #[ruma_api(body)]
    pub pdu: Box<RawJsonValue>,
}

/// Response type for the `send_knock` endpoint.
#[response]
pub struct Response {
    /// State events providing public room metadata.
    pub knock_room_state: Vec<RawStrippedState>,
}

impl Request {
    /// Creates a new `Request` with the given room ID, event ID and knock event.
    pub fn new(room_id: RoomId, event_id: EventId, pdu: Box<RawJsonValue>) -> Self {
        Self { room_id, event_id, pdu }
    }
}

impl Response {
    /// Creates a new `Response` with the given public room metadata state events.
    pub fn new(knock_room_state: Vec<RawStrippedState>) -> Self {
        Self { knock_room_state }
    }
}
