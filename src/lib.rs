// TODO
//!#[warn(missing_docs)]

use std::collections::HashMap;

use js_int::UInt;
use ruma_events::EventType;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Deserialize, Serialize)]
pub struct RoomV3Pdu {
    pub room_id: RoomId,
    pub sender: UserId,
    pub origin: String,
    pub origin_server_ts: UInt,

    // TODO: Replace with event content collection from ruma-events once that exists
    #[serde(rename = "type")]
    pub kind: EventType,
    pub content: JsonValue,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_key: Option<String>,
    pub prev_events: Vec<EventId>,
    pub depth: UInt,
    pub auth_events: Vec<EventId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redacts: Option<EventId>,
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub unsigned: serde_json::Map<String, JsonValue>,
    pub hashes: EventHash,
    pub signatures: HashMap<String, HashMap<String, String>>,
}

#[derive(Deserialize, Serialize)]
pub struct EventHash {
    pub sha256: String,
}
