//! [GET /_matrix/federation/v1/make_leave/{roomId}/{userId}](https://matrix.org/docs/spec/server_server/r0.1.4#get-matrix-federation-v1-make-leave-roomid-userid)

use std::time::SystemTime;

use ruma_api::ruma_api;
use ruma_events::{room::member::MemberEventContent, EventType};
use ruma_identifiers::{RoomId, RoomVersionId, ServerNameBox, UserId};
use ruma_serde::Raw;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata: {
        description: "Asks the receiving server to return information that the sending server will need to prepare a leave event to get out of the room.",
        name: "get_leave_event",
        method: GET,
        path: "/_matrix/federation/v1/make_leave/:room_id/:user_id",
        rate_limited: false,
        authentication: ServerSignatures,
    }

    request: {
        /// The room ID that is about to be left.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The user ID the leave event will be for.
        #[ruma_api(path)]
        pub user_id: &'a UserId,
    }

    response: {
        /// The version of the room where the server is trying to leave. If not provided, the room
        /// version is assumed to be either "1" or "2".
        pub room_version: Option<RoomVersionId>,

        /// An unsigned template event. Note that events have a different format depending on the
        /// room version - check the room version specification for precise event formats.
        pub event: EventTemplate,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with:
    /// * the room ID that is about to be left.
    /// * the user ID the leave event will be for.
    pub fn new(room_id: &'a RoomId, user_id: &'a UserId) -> Self {
        Self { room_id, user_id }
    }
}

impl Response {
    /// Creates a new `Response` with:
    /// * the version of the room where the server is trying to leave.
    /// * an unsigned template event.
    pub fn new(room_version: Option<RoomVersionId>, event: EventTemplate) -> Self {
        Self { room_version, event }
    }
}

/// An unsigned template event. Note that events have a different format depending on the room
/// version - check the room version specification for precise event formats.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct EventTemplate {
    /// The user ID of the leaving member.
    pub sender: UserId,

    /// The name of the resident homeserver.
    pub origin: ServerNameBox,

    /// A timestamp added by the resident homeserver.
    #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
    pub origin_server_ts: SystemTime,

    /// The value `m.room.member`.
    #[serde(rename = "type")]
    pub event_type: EventType,

    /// The user ID of the leaving member.
    pub state_key: String,

    /// The content of the event.
    pub content: Raw<MemberEventContent>,
}

impl EventTemplate {
    /// Creates a new `EventTemplate` with the given sender, origin, timestamp, state key and
    /// content.
    pub fn new(
        sender: UserId,
        origin: ServerNameBox,
        origin_server_ts: SystemTime,
        event_type: EventType,
        state_key: String,
        content: Raw<MemberEventContent>,
    ) -> Self {
        Self { sender, origin, origin_server_ts, event_type, state_key, content }
    }
}
