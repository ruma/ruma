//! `/v1/` ([spec])
//!
//! [spec]: https://spec.matrix.org/v1.4/server-server-api/#put_matrixfederationv1send_leaveroomideventid

use js_int::UInt;
use ruma_common::{
    api::{request, response, Metadata},
    events::{room::member::RoomMemberEventContent, StateEventType},
    metadata,
    serde::Raw,
    MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId, OwnedServerName, OwnedUserId,
};
use serde::{Deserialize, Serialize};

const METADATA: Metadata = metadata! {
    method: PUT,
    rate_limited: false,
    authentication: ServerSignatures,
    history: {
        1.0 => "/_matrix/federation/v1/send_leave/:room_id/:event_id",
    }
};

/// Request type for the `create_leave_event` endpoint.
#[request]
pub struct Request {
    /// The room ID that is about to be left.
    #[ruma_api(path)]
    pub room_id: OwnedRoomId,

    /// The event ID for the leave event.
    #[ruma_api(path)]
    pub event_id: OwnedEventId,

    /// The user ID of the leaving member.
    #[ruma_api(query)]
    pub sender: OwnedUserId,

    /// The name of the leaving homeserver.
    #[ruma_api(query)]
    pub origin: OwnedServerName,

    /// A timestamp added by the leaving homeserver.
    #[ruma_api(query)]
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The value `m.room.member`.
    #[ruma_api(query)]
    #[serde(rename = "type")]
    pub event_type: StateEventType,

    /// The user ID of the leaving member.
    #[ruma_api(query)]
    pub state_key: String,

    /// The content of the event.
    #[ruma_api(query)]
    pub content: Raw<RoomMemberEventContent>,

    /// This field must be present but is ignored; it may be 0.
    #[ruma_api(query)]
    pub depth: UInt,
}

/// Response type for the `create_leave_event` endpoint.
#[response]
#[derive(Default)]
pub struct Response {
    /// An empty object.
    ///
    /// Indicates that the event was accepted into the event graph.
    #[ruma_api(body)]
    #[serde(with = "crate::serde::v1_pdu")]
    pub empty: Empty,
}

/// Initial set of fields of `Request`.
///
/// This struct will not be updated even if additional fields are added to `Request` in a
/// new (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct RequestInit {
    /// The room ID that is about to be left.
    pub room_id: OwnedRoomId,

    /// The event ID for the leave event.
    pub event_id: OwnedEventId,

    /// The user ID of the leaving member.
    pub sender: OwnedUserId,

    /// The name of the leaving homeserver.
    pub origin: OwnedServerName,

    /// A timestamp added by the leaving homeserver.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The value `m.room.member`.
    pub event_type: StateEventType,

    /// The user ID of the leaving member.
    pub state_key: String,

    /// The content of the event.
    pub content: Raw<RoomMemberEventContent>,

    /// This field must be present but is ignored; it may be 0.
    pub depth: UInt,
}

impl From<RequestInit> for Request {
    /// Creates a new `Request` from `RequestInit`.
    fn from(init: RequestInit) -> Self {
        let RequestInit {
            room_id,
            event_id,
            sender,
            origin,
            origin_server_ts,
            event_type,
            state_key,
            content,
            depth,
        } = init;
        Self {
            room_id,
            event_id,
            sender,
            origin,
            origin_server_ts,
            event_type,
            state_key,
            content,
            depth,
        }
    }
}

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
