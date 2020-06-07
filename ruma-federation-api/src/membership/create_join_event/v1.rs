//! [PUT /_matrix/federation/v1/send_join/{roomId}/{eventId}](https://matrix.org/docs/spec/server_server/r0.1.3#put-matrix-federation-v1-send-join-roomid-eventid)

use std::{collections::BTreeMap, time::SystemTime};

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_events::EventType;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde_json::Value as JsonValue;

use super::RoomState;
use crate::{EventHash, RoomV3Pdu};

ruma_api! {
    metadata {
        description: "Send a join event to a resident server.",
        name: "create_join_event",
        method: PUT,
        path: "/_matrix/federation/v1/send_join/:room_id/:event_id",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The room ID that is about to be joined.
        #[ruma_api(path)]
        pub room_id: RoomId,
        /// The user ID the join event will be for.
        #[ruma_api(path)]
        pub event_id: EventId,

        /// The user id of the user who sent this event.
        pub sender: UserId,
        /// The `server_name` of the homeserver that created this event.
        pub origin: String,
        /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver
        /// of when this event was created.
        #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
        pub origin_server_ts: SystemTime,

        // TODO: Replace with event content collection from ruma-events once that exists
        /// The event's type.
        #[serde(rename = "type")]
        pub kind: EventType,
        /// The event's content.
        pub content: JsonValue,

        /// A key that determines which piece of room state the event represents.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub state_key: Option<String>,
        /// Event IDs for the most recent events in the room that the homeserver was
        /// aware of when it created this event.
        pub prev_events: Vec<EventId>,
        /// The maximum depth of the `prev_events`, plus one.
        pub depth: UInt,
        /// Event IDs for the authorization events that would allow this event to be
        /// in the room.
        pub auth_events: Vec<EventId>,
        /// For redaction events, the ID of the event being redacted.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub redacts: Option<EventId>,
        /// Additional data added by the origin server but not covered by the
        /// signatures.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub unsigned: BTreeMap<String, JsonValue>,
        /// Content hashes of the PDU.
        pub hashes: EventHash,
        /// Signatures for the PDU.
        pub signatures: BTreeMap<String, BTreeMap<String, String>>,
    }

    response {
        /// Full state of the room.
        #[ruma_api(body)]
        #[serde(with = "crate::serde::room_state")]
        pub room_state: RoomState,
    }
}

impl Request {
    /// Helper method to get event ID and PDU (with room ID) from the request
    /// parameters.
    pub fn into_id_and_v3_pdu(self) -> (EventId, RoomV3Pdu) {
        (
            self.event_id,
            RoomV3Pdu {
                room_id: self.room_id,
                sender: self.sender,
                origin: self.origin,
                origin_server_ts: self.origin_server_ts,
                kind: self.kind,
                content: self.content,
                state_key: self.state_key,
                prev_events: self.prev_events,
                depth: self.depth,
                auth_events: self.auth_events,
                redacts: self.redacts,
                unsigned: self.unsigned,
                hashes: self.hashes,
                signatures: self.signatures,
            },
        )
    }
}
