//! `/v2/` ([spec])
//!
//! [spec]: https://spec.matrix.org/latest/server-server-api/#put_matrixfederationv2send_joinroomideventid

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
        1.0 => "/_matrix/federation/v2/send_join/{room_id}/{event_id}",
    }
};

/// Request type for the `create_join_event` endpoint.
#[request]
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

    /// Indicates whether the calling server can accept a reduced response.
    ///
    /// If `true`, membership events are omitted from `state` and redundant events are omitted from
    /// `auth_chain` in the response.
    ///
    /// If the room to be joined has no `m.room.name` nor `m.room.canonical_alias` events in its
    /// current state, the resident server should determine the room members who would be
    /// included in the `m.heroes` property of the room summary as defined in the [Client-Server
    /// `/sync` response]. The resident server should include these members' membership events in
    /// the response `state` field, and include the auth chains for these membership events in
    /// the response `auth_chain` field.
    ///
    /// [Client-Server `/sync` response]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3sync
    #[ruma_api(query)]
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub omit_members: bool,
}

/// Response type for the `create_join_event` endpoint.
#[response]
pub struct Response {
    /// Full state of the room.
    #[ruma_api(body)]
    pub room_state: RoomState,
}

impl Request {
    /// Creates a new `Request` from the given room ID, event ID and PDU.
    pub fn new(room_id: OwnedRoomId, event_id: OwnedEventId, pdu: Box<RawJsonValue>) -> Self {
        Self { room_id, event_id, pdu, omit_members: false }
    }
}

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

    /// Whether `m.room.member` events have been omitted from `state`.
    ///
    /// Defaults to `false`.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub members_omitted: bool,

    /// The full set of authorization events that make up the state of the room,
    /// and their authorization events, recursively.
    ///
    /// If the request had `omit_members` set to `true`, then any events that are returned in
    /// `state` may be omitted from `auth_chain`, whether or not membership events are omitted
    /// from `state`.
    pub auth_chain: Vec<Box<RawJsonValue>>,

    /// The room state.
    ///
    /// If the request had `omit_members` set to `true`, events of type `m.room.member` may be
    /// omitted from the response to reduce the size of the response. If this is done,
    /// `members_omitted` must be set to `true`.
    pub state: Vec<Box<RawJsonValue>>,

    /// The signed copy of the membership event sent to other servers by the
    /// resident server, including the resident server's signature.
    ///
    /// Required if the room version supports restricted join rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<Box<RawJsonValue>>,

    /// A list of the servers active in the room (ie, those with joined members) before the join.
    ///
    /// Required if `members_omitted` is set to `true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers_in_room: Option<Vec<String>>,
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
        Self {
            origin,
            auth_chain: Vec::new(),
            state: Vec::new(),
            event: None,
            members_omitted: false,
            servers_in_room: None,
        }
    }

    #[cfg(feature = "unstable-unspecified")]
    /// Creates an empty `RoomState` with the given `origin`.
    ///
    /// Without the `unstable-unspecified` feature, this method takes a parameter for the origin.
    /// See [matrix-spec#374](https://github.com/matrix-org/matrix-spec/issues/374).
    pub fn new() -> Self {
        Self {
            auth_chain: Vec::new(),
            state: Vec::new(),
            event: None,
            members_omitted: false,
            servers_in_room: None,
        }
    }
}
