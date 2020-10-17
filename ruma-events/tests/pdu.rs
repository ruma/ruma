#![cfg(not(feature = "unstable-pre-spec"))]

use std::{
    collections::BTreeMap,
    time::{Duration, SystemTime},
};

use matches::assert_matches;
use ruma_events::{
    pdu::{EventHash, Pdu, PduStub, RoomV1Pdu, RoomV1PduStub, RoomV3Pdu, RoomV3PduStub},
    EventType,
};
use ruma_identifiers::{event_id, room_id, server_key_id, server_name, user_id};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn serialize_stub_as_v1() {
    let mut signatures = BTreeMap::new();
    let mut inner_signature = BTreeMap::new();
    inner_signature.insert(
        server_key_id!("ed25519:key_version"),
        "86BytesOfSignatureOfTheRedactedEvent".into(),
    );
    signatures.insert(server_name!("example.com"), inner_signature);

    let mut unsigned = BTreeMap::new();
    unsigned.insert("somekey".into(), json!({"a": 456}));

    let v1_stub = RoomV1PduStub {
        sender: user_id!("@sender:example.com"),
        origin: "matrix.org".into(),
        origin_server_ts: SystemTime::UNIX_EPOCH + Duration::from_millis(1_592_050_773_658),
        kind: EventType::RoomPowerLevels,
        content: json!({"testing": 123}),
        state_key: Some("state".into()),
        prev_events: vec![(
            event_id!("$previousevent:matrix.org"),
            EventHash { sha256: "123567".into() },
        )],
        depth: 2_u32.into(),
        auth_events: vec![(
            event_id!("$someauthevent:matrix.org"),
            EventHash { sha256: "21389CFEDABC".into() },
        )],
        redacts: Some(event_id!("$9654:matrix.org")),
        unsigned,
        hashes: EventHash { sha256: "1233543bABACDEF".into() },
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
        server_key_id!("ed25519:key_version"),
        "86BytesOfSignatureOfTheRedactedEvent".into(),
    );
    signatures.insert(server_name!("example.com"), inner_signature);

    let mut unsigned = BTreeMap::new();
    unsigned.insert("somekey".into(), json!({"a": 456}));

    let v3_stub = RoomV3PduStub {
        sender: user_id!("@sender:example.com"),
        origin: "matrix.org".into(),
        origin_server_ts: SystemTime::UNIX_EPOCH + Duration::from_millis(1_592_050_773_658),
        kind: EventType::RoomPowerLevels,
        content: json!({"testing": 123}),
        state_key: Some("state".into()),
        prev_events: vec![event_id!("$previousevent:matrix.org")],
        depth: 2_u32.into(),
        auth_events: vec![event_id!("$someauthevent:matrix.org")],
        redacts: Some(event_id!("$9654:matrix.org")),
        unsigned,
        hashes: EventHash { sha256: "1233543bABACDEF".into() },
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
fn deserialize_stub_as_v1() {
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
                "ed25519:key_version": "86BytesOfSignatureOfTheRedactedEvent"
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
            assert_eq!(v1_stub.auth_events.first().unwrap().0, event_id!("$abc123:matrix.org"));
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
                "ed25519:key_version": "86BytesOfSignatureOfTheRedactedEvent"
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
            assert_eq!(v3_stub.auth_events.first().unwrap(), &event_id!("$abc123:matrix.org"));
        }
    }
}

#[test]
fn serialize_pdu_as_v1() {
    let mut signatures = BTreeMap::new();
    let mut inner_signature = BTreeMap::new();
    inner_signature.insert(
        server_key_id!("ed25519:key_version"),
        "86BytesOfSignatureOfTheRedactedEvent".into(),
    );
    signatures.insert(server_name!("example.com"), inner_signature);

    let mut unsigned = BTreeMap::new();
    unsigned.insert("somekey".into(), json!({"a": 456}));

    let v1_pdu = RoomV1Pdu {
        room_id: room_id!("!n8f893n9:example.com"),
        event_id: event_id!("$somejoinevent:matrix.org"),
        sender: user_id!("@sender:example.com"),
        origin: "matrix.org".into(),
        origin_server_ts: SystemTime::UNIX_EPOCH + Duration::from_millis(1_592_050_773_658),
        kind: EventType::RoomPowerLevels,
        content: json!({"testing": 123}),
        state_key: Some("state".into()),
        prev_events: vec![(
            event_id!("$previousevent:matrix.org"),
            EventHash { sha256: "123567".into() },
        )],
        depth: 2_u32.into(),
        auth_events: vec![(
            event_id!("$someauthevent:matrix.org"),
            EventHash { sha256: "21389CFEDABC".into() },
        )],
        redacts: Some(event_id!("$9654:matrix.org")),
        unsigned,
        hashes: EventHash { sha256: "1233543bABACDEF".into() },
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
        server_key_id!("ed25519:key_version"),
        "86BytesOfSignatureOfTheRedactedEvent".into(),
    );
    signatures.insert(server_name!("example.com"), inner_signature);

    let mut unsigned = BTreeMap::new();
    unsigned.insert("somekey".into(), json!({"a": 456}));

    let v3_pdu = RoomV3Pdu {
        event_id: None,
        room_id: room_id!("!n8f893n9:example.com"),
        sender: user_id!("@sender:example.com"),
        origin: "matrix.org".into(),
        origin_server_ts: SystemTime::UNIX_EPOCH + Duration::from_millis(1_592_050_773_658),
        kind: EventType::RoomPowerLevels,
        content: json!({"testing": 123}),
        state_key: Some("state".into()),
        prev_events: vec![event_id!("$previousevent:matrix.org")],
        depth: 2_u32.into(),
        auth_events: vec![event_id!("$someauthevent:matrix.org")],
        redacts: Some(event_id!("$9654:matrix.org")),
        unsigned,
        hashes: EventHash { sha256: "1233543bABACDEF".into() },
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
                "ed25519:key_version": "86BytesOfSignatureOfTheRedactedEvent"
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
            assert_eq!(v1_pdu.auth_events.first().unwrap().0, event_id!("$abc123:matrix.org"));
            assert_eq!(
                v1_pdu.auth_events.first().unwrap().1.sha256,
                "Base64EncodedSha256HashesShouldBe43BytesLong"
            );
        }
        Pdu::RoomV3Pdu(_) => panic!("Matched V3 PDU"),
    }
}

#[cfg(not(feature = "unstable-pre-spec"))]
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
                "ed25519:key_version": "86BytesOfSignatureOfTheRedactedEvent"
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
            assert_eq!(v3_pdu.auth_events.first().unwrap(), &event_id!("$abc123:matrix.org"));
        }
    }
}

#[test]
fn convert_v1_stub_to_pdu() {
    let mut signatures = BTreeMap::new();
    let mut inner_signature = BTreeMap::new();
    inner_signature.insert(
        server_key_id!("ed25519:key_version"),
        "86BytesOfSignatureOfTheRedactedEvent".into(),
    );
    signatures.insert(server_name!("example.com"), inner_signature);

    let mut unsigned = BTreeMap::new();
    unsigned.insert("somekey".into(), json!({"a": 456}));

    let v1_stub = RoomV1PduStub {
        sender: user_id!("@sender:example.com"),
        origin: "matrix.org".into(),
        origin_server_ts: SystemTime::UNIX_EPOCH + Duration::from_millis(1_592_050_773_658),
        kind: EventType::RoomPowerLevels,
        content: json!({"testing": 123}),
        state_key: Some("state".into()),
        prev_events: vec![(
            event_id!("$previousevent:matrix.org"),
            EventHash { sha256: "123567".into() },
        )],
        depth: 2_u32.into(),
        auth_events: vec![(
            event_id!("$someauthevent:matrix.org"),
            EventHash { sha256: "21389CFEDABC".into() },
        )],
        redacts: Some(event_id!("$9654:matrix.org")),
        unsigned: (&unsigned).clone(),
        hashes: EventHash { sha256: "1233543bABACDEF".into() },
        signatures: (&signatures).clone(),
    };

    assert_matches!(
        v1_stub.into_v1_pdu(
            room_id!("!n8f893n9:example.com"),
            event_id!("$somejoinevent:matrix.org")
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
        } if room_id == room_id!("!n8f893n9:example.com")
            && event_id == event_id!("$somejoinevent:matrix.org")
            && sender == user_id!("@sender:example.com")
            && origin == "matrix.org"
            && origin_server_ts == SystemTime::UNIX_EPOCH + Duration::from_millis(1_592_050_773_658)
            && kind == EventType::RoomPowerLevels
            && content == json!({"testing": 123})
            && state_key == Some("state".into())
            && prev_events[0].0 == event_id!("$previousevent:matrix.org")
            && prev_events[0].1.sha256 == "123567"
            && depth == 2_u32.into()
            && auth_events.first().unwrap().0 == event_id!("$someauthevent:matrix.org")
            && auth_events.first().unwrap().1.sha256 == "21389CFEDABC"
            && redacts == Some(event_id!("$9654:matrix.org"))
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
        server_key_id!("ed25519:key_version"),
        "86BytesOfSignatureOfTheRedactedEvent".into(),
    );

    signatures.insert(server_name!("example.com"), inner_signature);

    let mut unsigned = BTreeMap::new();
    unsigned.insert("somekey".into(), json!({"a": 456}));

    let v3_stub = RoomV3PduStub {
        sender: user_id!("@sender:example.com"),
        origin: "matrix.org".into(),
        origin_server_ts: SystemTime::UNIX_EPOCH + Duration::from_millis(1_592_050_773_658),
        kind: EventType::RoomPowerLevels,
        content: json!({"testing": 123}),
        state_key: Some("state".into()),
        prev_events: vec![event_id!("$previousevent:matrix.org")],
        depth: 2_u32.into(),
        auth_events: vec![event_id!("$someauthevent:matrix.org")],
        redacts: Some(event_id!("$9654:matrix.org")),
        unsigned: (&unsigned).clone(),
        hashes: EventHash { sha256: "1233543bABACDEF".into() },
        signatures: (&signatures).clone(),
    };

    assert_matches!(
        v3_stub.into_v3_pdu(room_id!("!n8f893n9:example.com")),
        RoomV3Pdu {
            event_id: None,
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
        } if room_id == room_id!("!n8f893n9:example.com")
            && sender == user_id!("@sender:example.com")
            && origin == "matrix.org"
            && origin_server_ts == SystemTime::UNIX_EPOCH + Duration::from_millis(1_592_050_773_658)
            && kind == EventType::RoomPowerLevels
            && content == json!({"testing": 123})
            && state_key == Some("state".into())
            && prev_events == vec![event_id!("$previousevent:matrix.org")]
            && depth == 2_u32.into()
            && auth_events == vec![event_id!("$someauthevent:matrix.org")]
            && redacts == Some(event_id!("$9654:matrix.org"))
            && unsigned == (&unsigned).clone()
            && sha256 == "1233543bABACDEF"
            && signatures == (&signatures).clone()
    );
}
