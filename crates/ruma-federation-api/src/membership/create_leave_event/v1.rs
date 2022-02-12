//! [PUT /_matrix/federation/v1/send_leave/{roomId}/{eventId}](https://matrix.org/docs/spec/server_server/r0.1.4#put-matrix-federation-v1-send-leave-roomid-eventid)

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events::{room::member::RoomMemberEventContent, EventType};
use ruma_identifiers::{EventId, RoomId, ServerName, UserId};
use ruma_serde::Raw;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata: {
        description: "Submits a signed leave event to the receiving server for it to accept it into the room's graph.",
        name: "create_leave_event",
        method: PUT,
        stable: "/_matrix/federation/v1/send_leave/:room_id/:event_id",
        rate_limited: false,
        authentication: ServerSignatures,
        added: 1.0,
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
        pub content: Raw<RoomMemberEventContent>,

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

/// Initial set of fields of `Request`.
///
/// This struct will not be updated even if additional fields are added to `Request` in a
/// new (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct RequestInit<'a> {
    /// The room ID that is about to be left.
    pub room_id: &'a RoomId,

    /// The event ID for the leave event.
    pub event_id: &'a EventId,

    /// The user ID of the leaving member.
    pub sender: &'a UserId,

    /// The name of the leaving homeserver.
    pub origin: &'a ServerName,

    /// A timestamp added by the leaving homeserver.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The value `m.room.member`.
    pub event_type: EventType,

    /// The user ID of the leaving member.
    pub state_key: &'a str,

    /// The content of the event.
    pub content: Raw<RoomMemberEventContent>,

    /// This field must be present but is ignored; it may be 0.
    pub depth: UInt,
}

impl<'a> From<RequestInit<'a>> for Request<'a> {
    /// Creates a new `Request` from `RequestInit`.
    fn from(init: RequestInit<'a>) -> Self {
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
