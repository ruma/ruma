//! Types for persistent data unit schemas
//!
//! The differences between the `RoomV1Pdu` schema and the `RoomV3Pdu` schema are that the
//! `RoomV1Pdu` takes an `event_id` field (`RoomV3Pdu` does not), and `auth_events` and
//! `prev_events` take `Vec<(OwnedEventId, EventHash)>` rather than `Vec<OwnedEventId>` in
//! `RoomV3Pdu`.

use std::collections::BTreeMap;

use js_int::UInt;
use ruma_common::{
    MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId, OwnedUserId, ServerSignatures,
};
use serde::{
    de::{Error as _, IgnoredAny},
    Deserialize, Deserializer, Serialize,
};
use serde_json::{from_str as from_json_str, value::RawValue as RawJsonValue};

use super::TimelineEventType;

/// Enum for PDU schemas
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(untagged)]
pub enum Pdu {
    /// PDU for room versions 1 and 2.
    RoomV1Pdu(RoomV1Pdu),

    /// PDU for room versions 3 and above.
    RoomV3Pdu(RoomV3Pdu),
}

/// A 'persistent data unit' (event) for room versions 1 and 2.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::exhaustive_structs)]
pub struct RoomV1Pdu {
    /// Event ID for the PDU.
    pub event_id: OwnedEventId,

    /// The room this event belongs to.
    pub room_id: OwnedRoomId,

    /// The user id of the user who sent this event.
    pub sender: OwnedUserId,

    /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver
    /// of when this event was created.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    // TODO: Encode event type as content enum variant, like event enums do
    /// The event's type.
    #[serde(rename = "type")]
    pub kind: TimelineEventType,

    /// The event's content.
    pub content: Box<RawJsonValue>,

    /// A key that determines which piece of room state the event represents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_key: Option<String>,

    /// Event IDs for the most recent events in the room that the homeserver was
    /// aware of when it created this event.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub prev_events: Vec<(OwnedEventId, EventHash)>,

    /// The maximum depth of the `prev_events`, plus one.
    pub depth: UInt,

    /// Event IDs for the authorization events that would allow this event to be
    /// in the room.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub auth_events: Vec<(OwnedEventId, EventHash)>,

    /// For redaction events, the ID of the event being redacted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redacts: Option<OwnedEventId>,

    /// Additional data added by the origin server but not covered by the
    /// signatures.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub unsigned: BTreeMap<String, Box<RawJsonValue>>,

    /// Content hashes of the PDU.
    pub hashes: EventHash,

    /// Signatures for the PDU.
    pub signatures: ServerSignatures,
}

/// A 'persistent data unit' (event) for room versions 3 and beyond.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::exhaustive_structs)]
pub struct RoomV3Pdu {
    /// The room this event belongs to.
    pub room_id: OwnedRoomId,

    /// The user id of the user who sent this event.
    pub sender: OwnedUserId,

    /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver
    /// of when this event was created.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    // TODO: Encode event type as content enum variant, like event enums do
    /// The event's type.
    #[serde(rename = "type")]
    pub kind: TimelineEventType,

    /// The event's content.
    pub content: Box<RawJsonValue>,

    /// A key that determines which piece of room state the event represents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_key: Option<String>,

    /// Event IDs for the most recent events in the room that the homeserver was
    /// aware of when it created this event.
    pub prev_events: Vec<OwnedEventId>,

    /// The maximum depth of the `prev_events`, plus one.
    pub depth: UInt,

    /// Event IDs for the authorization events that would allow this event to be
    /// in the room.
    pub auth_events: Vec<OwnedEventId>,

    /// For redaction events, the ID of the event being redacted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redacts: Option<OwnedEventId>,

    /// Additional data added by the origin server but not covered by the
    /// signatures.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub unsigned: BTreeMap<String, Box<RawJsonValue>>,

    /// Content hashes of the PDU.
    pub hashes: EventHash,

    /// Signatures for the PDU.
    pub signatures: ServerSignatures,
}

/// Content hashes of a PDU.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct EventHash {
    /// The SHA-256 hash.
    pub sha256: String,
}

impl EventHash {
    /// Create a new `EventHash` with the given SHA256 hash.
    pub fn new(sha256: String) -> Self {
        Self { sha256 }
    }
}

impl<'de> Deserialize<'de> for Pdu {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct GetEventId {
            event_id: Option<IgnoredAny>,
        }

        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        if from_json_str::<GetEventId>(json.get()).map_err(D::Error::custom)?.event_id.is_some() {
            from_json_str(json.get()).map(Self::RoomV1Pdu).map_err(D::Error::custom)
        } else {
            from_json_str(json.get()).map(Self::RoomV3Pdu).map_err(D::Error::custom)
        }
    }
}
