//! Types for persistent data unit schemas
//!
//! The differences between the `RoomV1Pdu` schema and the `RoomV3Pdu` schema are
//! that the `RoomV1Pdu` takes an `event_id` field (`RoomV3Pdu` does not), and
//! `auth_events` and `prev_events` take `Vec<(EventId, EventHash)> rather than
//! `Vec<EventId>` in `RoomV3Pdu`.
//!
//! The stubbed versions of each PDU type remove the `event_id` field (if any)
//! and the `room_id` field for use in PDU templates.

use std::{
    collections::BTreeMap,
    time::{Duration, SystemTime},
};

use js_int::UInt;
use ruma_events::EventType;
use ruma_identifiers::{EventId, RoomId, UserId};
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
    /// Helper method to get PDU from a PDU stub.
    pub fn into_pdu(self, room_id: RoomId, event_id: EventId) -> Pdu {
        match self {
            PduStub::RoomV1PduStub(v1_stub) => {
                Pdu::RoomV1Pdu(v1_stub.into_v1_pdu(room_id, event_id))
            }
            PduStub::RoomV3PduStub(v3_stub) => Pdu::RoomV3Pdu(v3_stub.into_v3_pdu(room_id)),
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

impl RoomV1PduStub {
    /// Converts a V1 PDU stub into a full V1 PDU.
    pub fn into_v1_pdu(self, room_id: RoomId, event_id: EventId) -> RoomV1Pdu {
        RoomV1Pdu {
            event_id,
            room_id,
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
        }
    }
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

impl RoomV3PduStub {
    /// Converts a V3 PDU stub into a full V3 PDU.
    pub fn into_v3_pdu(self, room_id: RoomId) -> RoomV3Pdu {
        RoomV3Pdu {
            room_id,
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
        }
    }
}

/// Content hashes of a PDU.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventHash {
    /// The SHA-256 hash.
    pub sha256: String,
}

#[cfg(test)]
mod tests {
    use std::{convert::TryFrom, time::SystemTime};

    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::*;

    #[test]
    fn serialize_stub_as_v1() {
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
            origin_server_ts: SystemTime::UNIX_EPOCH + Duration::from_millis(1592050773658),
            kind: EventType::RoomPowerLevels,
            content: json!({"testing": 123}),
            state_key: Some("state".to_string()),
            prev_events: vec![(
                EventId::try_from("$previousevent:matrix.org").unwrap(),
                EventHash { sha256: "123567".to_string() },
            )],
            depth: 2_u32.into(),
            auth_events: vec![(
                EventId::try_from("$someauthevent:matrix.org").unwrap(),
                EventHash { sha256: "21389CFEDABC".to_string() },
            )],
            redacts: Some(EventId::try_from("$9654:matrix.org").unwrap()),
            unsigned,
            hashes: EventHash { sha256: "1233543bABACDEF".to_string() },
            signatures,
        };
        let pdu_stub = PduStub::RoomV1PduStub(v1_stub);
        let json = json!({
            "sender": "@sender:example.com",
            "origin": "matrix.org",
            "origin_server_ts": 1592050773658 as usize,
            "type": "m.room.power_levels",
            "content": {
                "testing": 123
            },
            "state_key": "state",
            "prev_events": [
                [ "$previousevent:matrix.org", {"sha256": "123567"} ]
            ],
            "depth": 2,
            "auth_events": [
                ["$someauthevent:matrix.org", {"sha256": "21389CFEDABC"}]
            ],
            "redacts": "$9654:matrix.org",
            "unsigned": {
                "somekey": { "a": 456 } },
            "hashes": { "sha256": "1233543bABACDEF" },
            "signatures": {
                "example.com": { "ed25519:key_version":"86BytesOfSignatureOfTheRedactedEvent" }
            }
        });

        assert_eq!(to_json_value(&pdu_stub).unwrap(), json);
    }

    #[test]
    fn serialize_stub_as_v3() {
        let mut signatures = BTreeMap::new();
        let mut inner_signature = BTreeMap::new();
        inner_signature.insert(
            "ed25519:key_version".to_string(),
            "86BytesOfSignatureOfTheRedactedEvent".to_string(),
        );
        signatures.insert("example.com".to_string(), inner_signature);

        let mut unsigned = BTreeMap::new();
        unsigned.insert("somekey".to_string(), json!({"a": 456}));

        let v3_stub = RoomV3PduStub {
            sender: UserId::try_from("@sender:example.com").unwrap(),
            origin: "matrix.org".to_string(),
            origin_server_ts: SystemTime::UNIX_EPOCH + Duration::from_millis(1592050773658),
            kind: EventType::RoomPowerLevels,
            content: json!({"testing": 123}),
            state_key: Some("state".to_string()),
            prev_events: vec![EventId::try_from("$previousevent:matrix.org").unwrap()],
            depth: 2_u32.into(),
            auth_events: vec![EventId::try_from("$someauthevent:matrix.org").unwrap()],
            redacts: Some(EventId::try_from("$9654:matrix.org").unwrap()),
            unsigned,
            hashes: EventHash { sha256: "1233543bABACDEF".to_string() },
            signatures,
        };
        let pdu_stub = PduStub::RoomV3PduStub(v3_stub);
        let json = json!({
            "sender": "@sender:example.com",
            "origin": "matrix.org",
            "origin_server_ts": 1592050773658 as usize,
            "type": "m.room.power_levels",
            "content": {
                "testing": 123
            },
            "state_key": "state",
            "prev_events": [ "$previousevent:matrix.org" ],
            "depth": 2,
            "auth_events": ["$someauthevent:matrix.org" ],
            "redacts": "$9654:matrix.org",
            "unsigned": {
                "somekey": { "a": 456 } },
            "hashes": { "sha256": "1233543bABACDEF" },
            "signatures": {
                "example.com": { "ed25519:key_version":"86BytesOfSignatureOfTheRedactedEvent" }
            }
        });

        assert_eq!(to_json_value(&pdu_stub).unwrap(), json);
    }

    #[test]
    fn test_deserialize_stub_as_v1() {
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
            }
            PduStub::RoomV3PduStub(_) => panic!("Matched V3 stub"),
        }
    }

    #[test]
    fn deserialize_stub_as_v3() {
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
            }
        }
    }

    #[test]
    fn serialize_pdu_as_v1() {
        let mut signatures = BTreeMap::new();
        let mut inner_signature = BTreeMap::new();
        inner_signature.insert(
            "ed25519:key_version".to_string(),
            "86BytesOfSignatureOfTheRedactedEvent".to_string(),
        );
        signatures.insert("example.com".to_string(), inner_signature);

        let mut unsigned = BTreeMap::new();
        unsigned.insert("somekey".to_string(), json!({"a": 456}));

        let v1_pdu = RoomV1Pdu {
            room_id: RoomId::try_from("!n8f893n9:example.com").unwrap(),
            event_id: EventId::try_from("$somejoinevent:matrix.org").unwrap(),
            sender: UserId::try_from("@sender:example.com").unwrap(),
            origin: "matrix.org".to_string(),
            origin_server_ts: SystemTime::UNIX_EPOCH + Duration::from_millis(1592050773658),
            kind: EventType::RoomPowerLevels,
            content: json!({"testing": 123}),
            state_key: Some("state".to_string()),
            prev_events: vec![(
                EventId::try_from("$previousevent:matrix.org").unwrap(),
                EventHash { sha256: "123567".to_string() },
            )],
            depth: 2_u32.into(),
            auth_events: vec![(
                EventId::try_from("$someauthevent:matrix.org").unwrap(),
                EventHash { sha256: "21389CFEDABC".to_string() },
            )],
            redacts: Some(EventId::try_from("$9654:matrix.org").unwrap()),
            unsigned,
            hashes: EventHash { sha256: "1233543bABACDEF".to_string() },
            signatures,
        };
        let pdu = Pdu::RoomV1Pdu(v1_pdu);
        let json = json!({
            "room_id": "!n8f893n9:example.com",
            "event_id": "$somejoinevent:matrix.org",
            "sender": "@sender:example.com",
            "origin": "matrix.org",
            "origin_server_ts": 1592050773658 as usize,
            "type": "m.room.power_levels",
            "content": {
                "testing": 123
            },
            "state_key": "state",
            "prev_events": [
                [ "$previousevent:matrix.org", {"sha256": "123567"} ]
            ],
            "depth": 2,
            "auth_events": [
                ["$someauthevent:matrix.org", {"sha256": "21389CFEDABC"}]
            ],
            "redacts": "$9654:matrix.org",
            "unsigned": {
                "somekey": { "a": 456 } },
            "hashes": { "sha256": "1233543bABACDEF" },
            "signatures": {
                "example.com": { "ed25519:key_version":"86BytesOfSignatureOfTheRedactedEvent" }
            }
        });

        assert_eq!(to_json_value(&pdu).unwrap(), json);
    }

    #[test]
    fn serialize_pdu_as_v3() {
        let mut signatures = BTreeMap::new();
        let mut inner_signature = BTreeMap::new();
        inner_signature.insert(
            "ed25519:key_version".to_string(),
            "86BytesOfSignatureOfTheRedactedEvent".to_string(),
        );
        signatures.insert("example.com".to_string(), inner_signature);

        let mut unsigned = BTreeMap::new();
        unsigned.insert("somekey".to_string(), json!({"a": 456}));

        let v3_pdu = RoomV3Pdu {
            room_id: RoomId::try_from("!n8f893n9:example.com").unwrap(),
            sender: UserId::try_from("@sender:example.com").unwrap(),
            origin: "matrix.org".to_string(),
            origin_server_ts: SystemTime::UNIX_EPOCH + Duration::from_millis(1592050773658),
            kind: EventType::RoomPowerLevels,
            content: json!({"testing": 123}),
            state_key: Some("state".to_string()),
            prev_events: vec![EventId::try_from("$previousevent:matrix.org").unwrap()],
            depth: 2_u32.into(),
            auth_events: vec![EventId::try_from("$someauthevent:matrix.org").unwrap()],
            redacts: Some(EventId::try_from("$9654:matrix.org").unwrap()),
            unsigned,
            hashes: EventHash { sha256: "1233543bABACDEF".to_string() },
            signatures,
        };
        let pdu_stub = Pdu::RoomV3Pdu(v3_pdu);
        let json = json!({
            "room_id": "!n8f893n9:example.com",
            "sender": "@sender:example.com",
            "origin": "matrix.org",
            "origin_server_ts": 1592050773658 as usize,
            "type": "m.room.power_levels",
            "content": {
                "testing": 123
            },
            "state_key": "state",
            "prev_events": [ "$previousevent:matrix.org" ],
            "depth": 2,
            "auth_events": ["$someauthevent:matrix.org" ],
            "redacts": "$9654:matrix.org",
            "unsigned": {
                "somekey": { "a": 456 } },
            "hashes": { "sha256": "1233543bABACDEF" },
            "signatures": {
                "example.com": { "ed25519:key_version":"86BytesOfSignatureOfTheRedactedEvent" }
            }
        });

        assert_eq!(to_json_value(&pdu_stub).unwrap(), json);
    }

    #[test]
    fn test_deserialize_pdu_as_v1() {
        let json = json!({
            "room_id": "!n8f893n9:example.com",
            "event_id": "$somejoinevent:matrix.org",
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
        let parsed = from_json_value::<Pdu>(json).unwrap();

        match parsed {
            Pdu::RoomV1Pdu(v1_pdu) => {
                assert_eq!(
                    v1_pdu.auth_events.first().unwrap().0,
                    EventId::try_from("$abc123:matrix.org").unwrap()
                );
                assert_eq!(
                    v1_pdu.auth_events.first().unwrap().1.sha256,
                    "Base64EncodedSha256HashesShouldBe43BytesLong"
                );
            }
            Pdu::RoomV3Pdu(_) => panic!("Matched V3 PDU"),
        }
    }

    #[test]
    fn deserialize_pdu_as_v3() {
        let json = json!({
            "room_id": "!n8f893n9:example.com",
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
        let parsed = from_json_value::<Pdu>(json).unwrap();

        match parsed {
            Pdu::RoomV1Pdu(_) => panic!("Matched V1 PDU"),
            Pdu::RoomV3Pdu(v3_pdu) => {
                assert_eq!(
                    v3_pdu.auth_events.first().unwrap(),
                    &EventId::try_from("$abc123:matrix.org").unwrap()
                );
            }
        }
    }

    #[test]
    fn convert_v1_stub_to_pdu() {
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
            origin_server_ts: SystemTime::UNIX_EPOCH + Duration::from_millis(1592050773658),
            kind: EventType::RoomPowerLevels,
            content: json!({"testing": 123}),
            state_key: Some("state".to_string()),
            prev_events: vec![(
                EventId::try_from("$previousevent:matrix.org").unwrap(),
                EventHash { sha256: "123567".to_string() },
            )],
            depth: 2_u32.into(),
            auth_events: vec![(
                EventId::try_from("$someauthevent:matrix.org").unwrap(),
                EventHash { sha256: "21389CFEDABC".to_string() },
            )],
            redacts: Some(EventId::try_from("$9654:matrix.org").unwrap()),
            unsigned: (&unsigned).clone(),
            hashes: EventHash { sha256: "1233543bABACDEF".to_string() },
            signatures: (&signatures).clone(),
        };

        let v1_pdu = RoomV1Pdu {
            room_id: RoomId::try_from("!n8f893n9:example.com").unwrap(),
            event_id: EventId::try_from("$somejoinevent:matrix.org").unwrap(),
            sender: UserId::try_from("@sender:example.com").unwrap(),
            origin: "matrix.org".to_string(),
            origin_server_ts: SystemTime::UNIX_EPOCH + Duration::from_millis(1592050773658),
            kind: EventType::RoomPowerLevels,
            content: json!({"testing": 123}),
            state_key: Some("state".to_string()),
            prev_events: vec![(
                EventId::try_from("$previousevent:matrix.org").unwrap(),
                EventHash { sha256: "123567".to_string() },
            )],
            depth: 2_u32.into(),
            auth_events: vec![(
                EventId::try_from("$someauthevent:matrix.org").unwrap(),
                EventHash { sha256: "21389CFEDABC".to_string() },
            )],
            redacts: Some(EventId::try_from("$9654:matrix.org").unwrap()),
            unsigned,
            hashes: EventHash { sha256: "1233543bABACDEF".to_string() },
            signatures,
        };
        todo!();
        /*
        assert_eq!(
            v1_stub.into_v1_pdu(
                RoomId::try_from("!n8f893n9:example.com").unwrap(),
                EventId::try_from("$somejoinevent:matrix.org").unwrap()
            ),
            v1_pdu
        );
        */
    }

    #[test]
    fn convert_v3_stub_to_pdu() {
        let mut signatures = BTreeMap::new();
        let mut inner_signature = BTreeMap::new();
        inner_signature.insert(
            "ed25519:key_version".to_string(),
            "86BytesOfSignatureOfTheRedactedEvent".to_string(),
        );
        signatures.insert("example.com".to_string(), inner_signature);

        let mut unsigned = BTreeMap::new();
        unsigned.insert("somekey".to_string(), json!({"a": 456}));

        let v3_stub = RoomV3PduStub {
            sender: UserId::try_from("@sender:example.com").unwrap(),
            origin: "matrix.org".to_string(),
            origin_server_ts: SystemTime::UNIX_EPOCH + Duration::from_millis(1592050773658),
            kind: EventType::RoomPowerLevels,
            content: json!({"testing": 123}),
            state_key: Some("state".to_string()),
            prev_events: vec![EventId::try_from("$previousevent:matrix.org").unwrap()],
            depth: 2_u32.into(),
            auth_events: vec![EventId::try_from("$someauthevent:matrix.org").unwrap()],
            redacts: Some(EventId::try_from("$9654:matrix.org").unwrap()),
            unsigned: (&unsigned).clone(),
            hashes: EventHash { sha256: "1233543bABACDEF".to_string() },
            signatures: (&signatures).clone(),
        };

        let v3_pdu = RoomV3Pdu {
            room_id: RoomId::try_from("!n8f893n9:example.com").unwrap(),
            sender: UserId::try_from("@sender:example.com").unwrap(),
            origin: "matrix.org".to_string(),
            origin_server_ts: SystemTime::UNIX_EPOCH + Duration::from_millis(1592050773658),
            kind: EventType::RoomPowerLevels,
            content: json!({"testing": 123}),
            state_key: Some("state".to_string()),
            prev_events: vec![EventId::try_from("$previousevent:matrix.org").unwrap()],
            depth: 2_u32.into(),
            auth_events: vec![EventId::try_from("$someauthevent:matrix.org").unwrap()],
            redacts: Some(EventId::try_from("$9654:matrix.org").unwrap()),
            unsigned,
            hashes: EventHash { sha256: "1233543bABACDEF".to_string() },
            signatures,
        };

        todo!();
        /*
        assert_eq!(
            v3_stub.into_v3_pdu(
                RoomId::try_from("!n8f893n9:example.com").unwrap(),
                EventId::try_from("$somejoinevent:matrix.org").unwrap()
            ),
            v3_pdu
        );
        */
    }
}
