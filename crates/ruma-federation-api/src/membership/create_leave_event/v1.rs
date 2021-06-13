//! [PUT /_matrix/federation/v1/send_leave/{roomId}/{eventId}](https://matrix.org/docs/spec/server_server/r0.1.4#put-matrix-federation-v1-send-leave-roomid-eventid)

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events::{room::member::MemberEventContent, EventType};
use ruma_identifiers::{EventId, RoomId, ServerName, UserId};
use ruma_serde::Raw;
use serde::{Deserialize, Serialize};

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
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The event ID for the leave event.
        #[ruma_api(path)]
        pub event_id: &'a EventId,

        /// The user ID of the leaving member.
        #[ruma_api(query)]
        pub sender: &'a UserId,

        /// The name of the leaving homeserver.
        #[ruma_api(query)]
        pub origin: &'a ServerName,

        /// A timestamp added by the leaving homeserver.
        #[ruma_api(query)]
        pub origin_server_ts: MilliSecondsSinceUnixEpoch,

        /// The value `m.room.member`.
        #[ruma_api(query)]
        #[serde(rename = "type")]
        pub event_type: EventType,

        /// The user ID of the leaving member.
        #[ruma_api(query)]
        pub state_key: &'a str,

        /// The content of the event.
        #[ruma_api(query)]
        pub content: Raw<MemberEventContent>,

        /// This field must be present but is ignored; it may be 0.
        #[ruma_api(query)]
        pub depth: UInt,
    }

    #[derive(Default)]
    response: {
        /// An empty object.
        ///
        /// Indicates that the event was accepted into the event graph.
        #[ruma_api(body)]
        #[serde(with = "crate::serde::v1_pdu")]
        pub empty: Empty,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with:
    /// * the room ID that is about to be left.
    /// * the event ID for the leave event.
    /// * the user ID of the leaving member.
    /// * the name of the leaving homeserver.
    /// * a timestamp added by the leaving homeserver.
    /// * the value `m.room.member`.
    /// * the user ID of the leaving member.
    /// * the content of the event.
    /// * this field must be present but is ignored; it may be 0.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        room_id: &'a RoomId,
        event_id: &'a EventId,
        sender: &'a UserId,
        origin: &'a ServerName,
        origin_server_ts: MilliSecondsSinceUnixEpoch,
        event_type: EventType,
        state_key: &'a str,
        content: Raw<MemberEventContent>,
        depth: UInt,
    ) -> Self {
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
