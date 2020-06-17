use std::{
    collections::BTreeMap,
    convert::TryFrom,
    time::{Duration, SystemTime},
};

use matches::assert_matches;
use ruma_events::{
    pdu::{EventHash, Pdu, PduStub, RoomV1Pdu, RoomV1PduStub, RoomV3Pdu, RoomV3PduStub},
    EventType,
};
use ruma_identifiers::{EventId, RoomId, UserId};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

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
        origin_server_ts: SystemTime::UNIX_EPOCH + Duration::from_millis(1_592_050_773_658),
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
        "origin_server_ts": 1_592_050_773_658 as usize,
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
        origin_server_ts: SystemTime::UNIX_EPOCH + Duration::from_millis(1_592_050_773_658),
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
        "origin_server_ts": 1_592_050_773_658 as usize,
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
        "origin_server_ts": 1_234_567_890,
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
        "origin_server_ts": 1_234_567_890,
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
        origin_server_ts: SystemTime::UNIX_EPOCH + Duration::from_millis(1_592_050_773_658),
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
        "origin_server_ts": 1_592_050_773_658 as usize,
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
        origin_server_ts: SystemTime::UNIX_EPOCH + Duration::from_millis(1_592_050_773_658),
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
        "origin_server_ts": 1_592_050_773_658 as usize,
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
        "origin_server_ts": 1_234_567_890,
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
        "origin_server_ts": 1_234_567_890,
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
        origin_server_ts: SystemTime::UNIX_EPOCH + Duration::from_millis(1_592_050_773_658),
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

    assert_matches!(
        v1_stub.into_v1_pdu(
            RoomId::try_from("!n8f893n9:example.com").unwrap(),
            EventId::try_from("$somejoinevent:matrix.org").unwrap()
        ),
        RoomV1Pdu {
            room_id,
            event_id,
            sender,
            origin,
            origin_server_ts,
            kind,
            content,
            state_key,
            prev_events,
            depth,
            auth_events,
            redacts,
            unsigned,
            hashes: EventHash { sha256 },
            signatures,
        } if room_id == RoomId::try_from("!n8f893n9:example.com").unwrap()
            && event_id == EventId::try_from("$somejoinevent:matrix.org").unwrap()
            && sender == UserId::try_from("@sender:example.com").unwrap()
            && origin == "matrix.org"
            && origin_server_ts == SystemTime::UNIX_EPOCH + Duration::from_millis(1_592_050_773_658)
            && kind == EventType::RoomPowerLevels
            && content == json!({"testing": 123})
            && state_key == Some("state".to_string())
            && prev_events[0].0 == EventId::try_from("$previousevent:matrix.org").unwrap()
            && prev_events[0].1.sha256 == "123567"
            && depth == 2_u32.into()
            && auth_events.first().unwrap().0 == EventId::try_from("$someauthevent:matrix.org").unwrap()
            && auth_events.first().unwrap().1.sha256 == "21389CFEDABC"
            && redacts == Some(EventId::try_from("$9654:matrix.org").unwrap())
            && unsigned == (&unsigned).clone()
            && sha256 == "1233543bABACDEF"
            && signatures == (&signatures).clone()
    );
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
        origin_server_ts: SystemTime::UNIX_EPOCH + Duration::from_millis(1_592_050_773_658),
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

    assert_matches!(
        v3_stub.into_v3_pdu(RoomId::try_from("!n8f893n9:example.com").unwrap()),
        RoomV3Pdu {
            room_id,
            sender,
            origin,
            origin_server_ts,
            kind,
            content,
            state_key,
            prev_events,
            depth,
            auth_events,
            redacts,
            unsigned,
            hashes: EventHash { sha256 },
            signatures,
        } if room_id == RoomId::try_from("!n8f893n9:example.com").unwrap()
            && sender == UserId::try_from("@sender:example.com").unwrap()
            && origin == "matrix.org"
            && origin_server_ts == SystemTime::UNIX_EPOCH + Duration::from_millis(1_592_050_773_658)
            && kind == EventType::RoomPowerLevels
            && content == json!({"testing": 123})
            && state_key == Some("state".to_string())
            && prev_events == vec![EventId::try_from("$previousevent:matrix.org").unwrap()]
            && depth == 2_u32.into()
            && auth_events == vec![EventId::try_from("$someauthevent:matrix.org").unwrap()]
            && redacts == Some(EventId::try_from("$9654:matrix.org").unwrap())
            && unsigned == (&unsigned).clone()
            && sha256 == "1233543bABACDEF"
            && signatures == (&signatures).clone()
    );
}
