//! (De)serializable types for the Matrix Federation API.

#![warn(missing_docs)]

use std::{collections::BTreeMap, time::SystemTime};

use ::serde::{Deserialize, Serialize};
use js_int::UInt;
use ruma_events::EventType;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde_json::Value as JsonValue;

mod serde;

pub mod directory;
pub mod discovery;
pub mod membership;
pub mod query;
pub mod transactions;

/// A 'persistent data unit' (event) for room versions 3 and beyond.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoomV3Pdu {
    /// The room this event belongs to.
    pub room_id: RoomId,
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

/// Content hashes of a PDU.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventHash {
    /// The SHA-256 hash.
    pub sha256: String,
}
