//! Types for persistent data unit schemas
//!
//! The differences between the `RoomV1Pdu` schema and the `RoomV3Pdu` schema are
//! that the `RoomV1Pdu` takes an `event_id` field (`RoomV3Pdu` does not), and
//! `auth_events` and `prev_events` take `Vec<(EventId, EventHash)> rather than
//! `Vec<EventId>` in `RoomV3Pdu`.
//!
//! The stubbed versions of each PDU type remove the `event_id` field (if any)
//! and the `room_id` field for use in PDU templates.

use std::{collections::BTreeMap, time::SystemTime};

use js_int::UInt;
use ruma_events::EventType;
use ruma_identifiers::{EventId, RoomId, RoomVersionId, ServerNameBox, ServerSigningKeyId, UserId};
use ruma_serde::Raw;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Enum for PDU schemas
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Pdu {
    /// PDU for room versions 1 and 2.
    RoomV1Pdu(RoomV1Pdu),

    /// PDU for room versions 3 and above.
    RoomV3Pdu(RoomV3Pdu),
}

/// A 'persistent data unit' (event) for room versions 1 and 2.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoomV1Pdu {
    /// Event ID for the PDU.
    pub event_id: EventId,

    /// The room this event belongs to.
    pub room_id: RoomId,

    /// The user id of the user who sent this event.
    pub sender: UserId,

    #[cfg(not(feature = "unstable-pre-spec"))]
    /// The `server_name` of the homeserver that created this event.
    pub origin: String,

    /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver
    /// of when this event was created.
    #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
    pub origin_server_ts: SystemTime,

    // TODO: Encode event type as content enum variant, like event enums do
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub prev_events: Vec<(EventId, EventHash)>,

    /// The maximum depth of the `prev_events`, plus one.
    pub depth: UInt,

    /// Event IDs for the authorization events that would allow this event to be
    /// in the room.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub auth_events: Vec<(EventId, EventHash)>,

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
    pub signatures: BTreeMap<ServerNameBox, BTreeMap<ServerSigningKeyId, String>>,
}

/// A 'persistent data unit' (event) for room versions 3 and beyond.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoomV3Pdu {
    /// The room this event belongs to.
    pub room_id: RoomId,

    /// The user id of the user who sent this event.
    pub sender: UserId,

    #[cfg(not(feature = "unstable-pre-spec"))]
    /// The `server_name` of the homeserver that created this event.
    pub origin: String,

    /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver
    /// of when this event was created.
    #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
    pub origin_server_ts: SystemTime,

    // TODO: Encode event type as content enum variant, like event enums do
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
    pub signatures: BTreeMap<ServerNameBox, BTreeMap<ServerSigningKeyId, String>>,
}

/// Content hashes of a PDU.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventHash {
    /// The SHA-256 hash.
    pub sha256: String,
}

/// The PDU type that is used by the server. A `ServerPdu` is never sent or received
/// it is only to make dealing with PDU's during state resolution simpler.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct ServerPdu {
    /// Event ID for the PDU. When handling PDU's the server needs to always know
    /// the event_id for every PDU regardless of room version.
    pub event_id: EventId,

    /// The room this event belongs to.
    pub room_id: RoomId,

    /// The user id of the user who sent this event.
    pub sender: UserId,

    /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver
    /// of when this event was created.
    #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
    pub origin_server_ts: SystemTime,

    /// The event's type.
    #[serde(rename = "type")]
    pub kind: EventType,

    /// The content of this event.
    pub content: serde_json::Value,

    /// The state key if this event is a state event or None.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_key: Option<String>,

    /// The events that preceded this event.
    pub prev_events: Vec<EventId>,

    /// The maximum depth of the `prev_events`, plus one.
    pub depth: UInt,

    /// Event IDs for the authorization events that would allow this event to be
    /// in the room.
    pub auth_events: Vec<EventId>,

    /// If this event is a redaction event then this is the ID of the event
    /// being redacted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redacts: Option<EventId>,

    /// Content that is not part of the hash or signature of this event.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub unsigned: BTreeMap<String, JsonValue>,

    /// The content hash of this event.
    pub hashes: EventHash,

    /// Signatures for this event.
    pub signatures: BTreeMap<ServerNameBox, BTreeMap<ServerSigningKeyId, String>>,
}

impl ServerPdu {
    /// Convert a `ServerPdu` into a sendable PDU following the spec
    /// for room version 3 and above.
    pub fn into_pdu_v3_above(
        self,
        #[cfg(not(feature = "unstable-pre-spec"))] origin: String,
    ) -> Pdu {
        Pdu::RoomV3Pdu(RoomV3Pdu {
            room_id: self.room_id,
            sender: self.sender,
            #[cfg(not(feature = "unstable-pre-spec"))]
            origin,
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
        })
    }

    /// Convert `ServerPdu` to a PDU that can be sent over federation.
    pub fn convert_to_outgoing_federation_event(mut self) -> Raw<Pdu> {
        self.unsigned.remove("transaction_id");

        let mut pdu_json = serde_json::to_value(self).expect("TODO handle errors");
        pdu_json.as_object_mut().expect("json is object").remove("event_id");

        serde_json::from_value::<Raw<_>>(pdu_json).expect("Raw::from_value always works")
    }

    /// Convert a `ServerPdu` into a sendable PDU following the spec
    /// for rooms less than version 3.
    pub fn to_pdu_less_v3(
        self,
        prev: Vec<(EventId, EventHash)>,
        auth: Vec<(EventId, EventHash)>,
    ) -> Pdu {
        todo!()
    }
}
