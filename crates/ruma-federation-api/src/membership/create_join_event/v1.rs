//! `/v1/` ([spec])
//!
//! [spec]: https://spec.matrix.org/latest/server-server-api/#put_matrixfederationv1send_joinroomideventid

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
        1.0 => "/_matrix/federation/v1/send_join/:room_id/:event_id",
        1.0 => deprecated,
    }
};

/// Request type for the `create_join_event` endpoint.
#[request]
#[deprecated = "Since Matrix Server-Server API r0.1.4. Use the v2 endpoint instead."]
pub struct Request {
    /// The room ID that is about to be joined.
    ///
    /// Do not use this. Instead, use the `room_id` field inside the PDU.
    #[ruma_api(path)]
    pub room_id: OwnedRoomId,

    /// The event ID for the join event.
    #[ruma_api(path)]
    pub event_id: OwnedEventId,

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
    pub fn new(room_id: OwnedRoomId, event_id: OwnedEventId, pdu: Box<RawJsonValue>) -> Self {
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
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomState {
    #[cfg(not(feature = "unstable-unspecified"))]
    /// The resident server's DNS name.
    pub origin: String,

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

#[cfg(feature = "unstable-unspecified")]
impl Default for RoomState {
    fn default() -> Self {
        Self::new()
    }
}

impl RoomState {
    #[cfg(not(feature = "unstable-unspecified"))]
    /// Creates an empty `RoomState` with the given `origin`.
    ///
    /// With the `unstable-unspecified` feature, this method doesn't take any parameters.
    /// See [matrix-spec#374](https://github.com/matrix-org/matrix-spec/issues/374).
    pub fn new(origin: String) -> Self {
        Self { origin, auth_chain: Vec::new(), state: Vec::new(), event: None }
    }

    #[cfg(feature = "unstable-unspecified")]
    /// Creates an empty `RoomState` with the given `origin`.
    ///
    /// Without the `unstable-unspecified` feature, this method takes a parameter for the origin.
    /// See [matrix-spec#374](https://github.com/matrix-org/matrix-spec/issues/374).
    pub fn new() -> Self {
        Self { auth_chain: Vec::new(), state: Vec::new(), event: None }
    }
}
