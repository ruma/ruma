//! `/v1/` ([spec])
//!
//! [spec]: https://spec.matrix.org/latest/server-server-api/#put_matrixfederationv1send_joinroomideventid

use ruma_common::{
    EventId, RoomId,
    api::{request, response},
    metadata,
};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;

use crate::authentication::ServerSignatures;

metadata! {
    method: PUT,
    rate_limited: false,
    authentication: ServerSignatures,
    path: "/_matrix/federation/v1/send_join/{room_id}/{event_id}",
}

/// Request type for the `create_join_event` endpoint.
#[request]
#[deprecated = "Since Matrix Server-Server API r0.1.4. Use the v2 endpoint instead."]
pub struct Request {
    /// The room ID that is about to be joined.
    ///
    /// Do not use this. Instead, use the `room_id` field inside the PDU.
    #[ruma_api(path)]
    pub room_id: RoomId,

    /// The event ID for the join event.
    #[ruma_api(path)]
    pub event_id: EventId,

    /// The PDU.
    #[ruma_api(body)]
    pub pdu: Box<RawJsonValue>,
}

/// Response type for the `create_join_event` endpoint.
#[response]
#[deprecated = "Since Matrix Server-Server API r0.1.4. Use the v2 endpoint instead."]
pub struct Response {
    /// Full state and auth chain of the room prior to the join event.
    #[ruma_api(body)]
    #[serde(with = "crate::serde::v1_pdu")]
    pub room_state: RoomState,
}

#[allow(deprecated)]
impl Request {
    /// Creates a new `Request` from the given room ID, event ID and PDU.
    pub fn new(room_id: RoomId, event_id: EventId, pdu: Box<RawJsonValue>) -> Self {
        Self { room_id, event_id, pdu }
    }
}

#[allow(deprecated)]
impl Response {
    /// Creates a new `Response` with the given room state.
    pub fn new(room_state: RoomState) -> Self {
        Self { room_state }
    }
}

/// Full state of the room.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RoomState {
    /// The full set of authorization events that make up the state of the room,
    /// and their authorization events, recursively.
    pub auth_chain: Vec<Box<RawJsonValue>>,

    /// The room state.
    pub state: Vec<Box<RawJsonValue>>,

    /// The signed copy of the membership event sent to other servers by the
    /// resident server, including the resident server's signature.
    ///
    /// Required if the room version supports restricted join rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<Box<RawJsonValue>>,
}

impl RoomState {
    /// Creates an empty `RoomState`.
    pub fn new() -> Self {
        Self::default()
    }
}
