//! Types for persistent data unit schemas

use std::{collections::BTreeMap, time::SystemTime};

use ::serde::{Deserialize, Serialize};
use js_int::UInt;
use ruma_events::EventType;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde_json::Value as JsonValue;

/// Enum for PDU schemas
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    pub signatures: BTreeMap<String, BTreeMap<String, String>>,
}
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

/// PDU type without event and room IDs.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PduStub {
    /// Stub for PDUs of room version 1 and 2.
    RoomV1PduStub(RoomV1PduStub),

    /// Stub for PDUs of room versions 3 and above.
    RoomV3PduStub(RoomV3PduStub),
}

impl PduStub {
    /// Helper method to get event ID and PDU (with room ID) from the request
    /// parameters.
    pub fn into_pdu(self, room_id: RoomId, event_id: EventId) -> Pdu {
        match self {
            PduStub::RoomV1PduStub(v1_stub) => Pdu::RoomV1Pdu(RoomV1Pdu {
                event_id,
                room_id,
                sender: v1_stub.sender,
                origin: v1_stub.origin,
                origin_server_ts: v1_stub.origin_server_ts,
                kind: v1_stub.kind,
                content: v1_stub.content,
                state_key: v1_stub.state_key,
                prev_events: v1_stub.prev_events,
                depth: v1_stub.depth,
                auth_events: v1_stub.auth_events,
                redacts: v1_stub.redacts,
                unsigned: v1_stub.unsigned,
                hashes: v1_stub.hashes,
                signatures: v1_stub.signatures,
            }),
            PduStub::RoomV3PduStub(v3_stub) => Pdu::RoomV3Pdu(RoomV3Pdu {
                room_id,
                sender: v3_stub.sender,
                origin: v3_stub.origin,
                origin_server_ts: v3_stub.origin_server_ts,
                kind: v3_stub.kind,
                content: v3_stub.content,
                state_key: v3_stub.state_key,
                prev_events: v3_stub.prev_events,
                depth: v3_stub.depth,
                auth_events: v3_stub.auth_events,
                redacts: v3_stub.redacts,
                unsigned: v3_stub.unsigned,
                hashes: v3_stub.hashes,
                signatures: v3_stub.signatures,
            }),
        }
    }
}

/// Stub for PDUs of room version 1 and 2.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoomV1PduStub {
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
    pub prev_events: Vec<(EventId, EventHash)>,

    /// The maximum depth of the `prev_events`, plus one.
    pub depth: UInt,

    /// Event IDs for the authorization events that would allow this event to be
    /// in the room.
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
    pub signatures: BTreeMap<String, BTreeMap<String, String>>,
}

/// Stub for PDUs of room versions 3 and above.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoomV3PduStub {
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

#[cfg(test)]
mod tests {

    use std::{convert::TryFrom, time::SystemTime};

    use serde_json::{from_value as from_json_value, json};

    use super::*;
    #[test]
    fn test_serialize_pdu_stub() {

        let mut signatures = BTreeMap::new();
        let mut inner_signature = BTreeMap::new();
        inner_signature.insert(
            "ed25519:key_version".to_string(),
            "86BytesOfSignatureOfTheRedactedEvent".to_string(),
        );
        signatures.insert("example.com".to_string(), inner_signature);

        let mut unsigned = BTreeMap::new();
        unsigned.insert("somekey".to_string(), json!({"a": 456}));

        let v1_stub = RoomV1PduStub {
            sender: UserId::try_from("@sender:example.com").unwrap(),
            origin: "matrix.org".to_string(),
            origin_server_ts: SystemTime::now(),
            kind: EventType::RoomPowerLevels,
            content: json!({"testing": 123}),
            state_key: Some("state".to_string()),
            prev_events: vec![(
                EventId::try_from("$previousevent:matrix.org").unwrap(),
                EventHash {
                    sha256: "123567".to_string(),
                },
            )],
            depth: 2_u32.into(),
            auth_events: vec![(
                EventId::try_from("$someauthevent:matrix.org").unwrap(),
                EventHash {
                    sha256: "21389CFEDABC".to_string(),
                },
            )],
            redacts: Some(EventId::try_from("$9654:matrix.org").unwrap()),
            unsigned,
            hashes: EventHash {
                sha256: "1233543bABACDEF".to_string(),
            },
            signatures,
        };
        let pdu_stub = PduStub::RoomV1PduStub(v1_stub);
        // TODO: Test for a real value
        assert!(true);
    }

    #[test]
    fn test_deserialize_v1_stub() {
        let json = json!({
            "auth_events": [
                [
                    "$abc123:matrix.org",
                    {
                        "sha256": "Base64EncodedSha256HashesShouldBe43BytesLong"
                    }
                ]
            ],
            "content": {
                "key": "value"
            },
            "depth": 12,
            "event_id": "$a4ecee13e2accdadf56c1025:example.com",
            "hashes": {
                "sha256": "ThisHashCoversAllFieldsInCaseThisIsRedacted"
            },
            "origin": "matrix.org",
            "origin_server_ts": 1234567890,
            "prev_events": [
                [
                    "$abc123:matrix.org",
                    {
                        "sha256": "Base64EncodedSha256HashesShouldBe43BytesLong"
                    }
                ]
            ],
            "redacts": "$def456:matrix.org",
            "room_id": "!abc123:matrix.org",
            "sender": "@someone:matrix.org",
            "signatures": {
                "example.com": {
                    "ed25519:key_version:": "86BytesOfSignatureOfTheRedactedEvent"
                }
            },
            "state_key": "my_key",
            "type": "m.room.message",
            "unsigned": {
                "key": "value"
            }
        });
        let parsed = from_json_value::<PduStub>(json).unwrap();

        match parsed {
            PduStub::RoomV1PduStub(v1_stub) => {
                assert_eq!(
                    v1_stub.auth_events.first().unwrap().0,
                    EventId::try_from("$abc123:matrix.org").unwrap()
                );
                assert_eq!(
                    v1_stub.auth_events.first().unwrap().1.sha256,
                    "Base64EncodedSha256HashesShouldBe43BytesLong"
                );
            },
            PduStub::RoomV3PduStub(_) => panic!("Matched V3 stub"),
        }
    }
    
    #[test]
    fn test_deserialize_v3_stub() {
        let json = json!({
            "auth_events": [
                    "$abc123:matrix.org"
            ],
            "content": {
                "key": "value"
            },
            "depth": 12,
            "event_id": "$a4ecee13e2accdadf56c1025:example.com",
            "hashes": {
                "sha256": "ThisHashCoversAllFieldsInCaseThisIsRedacted"
            },
            "origin": "matrix.org",
            "origin_server_ts": 1234567890,
            "prev_events": [
                    "$abc123:matrix.org"
            ],
            "redacts": "$def456:matrix.org",
            "room_id": "!abc123:matrix.org",
            "sender": "@someone:matrix.org",
            "signatures": {
                "example.com": {
                    "ed25519:key_version:": "86BytesOfSignatureOfTheRedactedEvent"
                }
            },
            "state_key": "my_key",
            "type": "m.room.message",
            "unsigned": {
                "key": "value"
            }
        });
        let parsed = from_json_value::<PduStub>(json).unwrap();

        match parsed {
            PduStub::RoomV1PduStub(_) => panic!("Matched V1 stub"),
            PduStub::RoomV3PduStub(v3_stub) => {
                assert_eq!(
                    v3_stub.auth_events.first().unwrap(),
                    &EventId::try_from("$abc123:matrix.org").unwrap()
                );
            },
        }
    }
}