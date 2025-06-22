//! `/v1/` ([spec])
//!
//! [spec]: https://spec.matrix.org/latest/server-server-api/#put_matrixfederationv1send_leaveroomideventid

use ruma_common::{
    api::{request, response, Metadata},
    metadata, OwnedEventId, OwnedRoomId,
};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;

const METADATA: Metadata = metadata! {
    method: PUT,
    rate_limited: false,
    authentication: ServerSignatures,
    history: {
        1.0 => "/_matrix/federation/v1/send_leave/{room_id}/{event_id}",
        1.0 => deprecated,
    }
};

/// Request type for the `create_leave_event` endpoint.
#[request]
#[deprecated = "Since Matrix Server-Server API r0.1.4. Use the v2 endpoint instead."]
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
#[deprecated = "Since Matrix Server-Server API r0.1.4. Use the v2 endpoint instead."]
pub struct Response {
    /// An empty object.
    ///
    /// Indicates that the event was accepted into the event graph.
    #[ruma_api(body)]
    #[serde(with = "crate::serde::v1_pdu")]
    pub empty: Empty,
}

#[allow(deprecated)]
impl Request {
    /// Creates a new `Request` from the given room ID, event ID and PDU.
    pub fn new(room_id: OwnedRoomId, event_id: OwnedEventId, pdu: Box<RawJsonValue>) -> Self {
        Self { room_id, event_id, pdu }
    }
}

#[allow(deprecated)]
impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self { empty: Empty {} }
    }
}

/// An empty object.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[allow(clippy::exhaustive_structs)]
pub struct Empty {}

impl Empty {
    /// Create a new `Empty`.
    pub fn new() -> Self {
        Self {}
    }
}
