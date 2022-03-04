#![cfg(all(feature = "unstable-pdu", not(feature = "unstable-pre-spec")))]

use std::{collections::BTreeMap, convert::TryInto};

use ruma_common::{
    event_id,
    events::{
        pdu::{EventHash, Pdu, RoomV1Pdu, RoomV3Pdu},
        EventType,
    },
    room_id, server_name, server_signing_key_id, user_id, MilliSecondsSinceUnixEpoch,
};
use serde_json::{
    from_value as from_json_value, json, to_value as to_json_value,
    value::to_raw_value as to_raw_json_value,
};

#[test]
fn serialize_pdu_as_v1() {
    let mut signatures = BTreeMap::new();
    let mut inner_signature = BTreeMap::new();
    inner_signature.insert(
        server_signing_key_id!("ed25519:key_version").to_owned(),
        "86BytesOfSignatureOfTheRedactedEvent".into(),
    );
    signatures.insert(server_name!("example.com").to_owned(), inner_signature);

    let mut unsigned = BTreeMap::new();
    unsigned.insert("somekey".into(), to_raw_json_value(&json!({ "a": 456 })).unwrap());

    let v1_pdu = RoomV1Pdu {
        room_id: room_id!("!n8f893n9:example.com").to_owned(),
        event_id: event_id!("$somejoinevent:matrix.org").to_owned(),
        sender: user_id!("@sender:example.com").to_owned(),
        origin: "matrix.org".into(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(1_592_050_773_658_u64.try_into().unwrap()),
        kind: EventType::RoomPowerLevels,
        content: to_raw_json_value(&json!({ "testing": 123 })).unwrap(),
        state_key: Some("state".into()),
        prev_events: vec![(
            event_id!("$previousevent:matrix.org").to_owned(),
            EventHash::new("123567".into()),
        )],
        depth: 2_u32.into(),
        auth_events: vec![(
            event_id!("$someauthevent:matrix.org").to_owned(),
            EventHash::new("21389CFEDABC".into()),
        )],
        redacts: Some(event_id!("$9654:matrix.org").to_owned()),
        unsigned,
        hashes: EventHash::new("1233543bABACDEF".into()),
        signatures,
    };
    let pdu = Pdu::RoomV1Pdu(v1_pdu);
    let json = json!({
        "room_id": "!n8f893n9:example.com",
        "event_id": "$somejoinevent:matrix.org",
        "sender": "@sender:example.com",
        "origin": "matrix.org",
        "origin_server_ts": 1_592_050_773_658_u64,
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
        server_signing_key_id!("ed25519:key_version").to_owned(),
        "86BytesOfSignatureOfTheRedactedEvent".into(),
    );
    signatures.insert(server_name!("example.com").to_owned(), inner_signature);

    let mut unsigned = BTreeMap::new();
    unsigned.insert("somekey".into(), to_raw_json_value(&json!({ "a": 456 })).unwrap());

    let v3_pdu = RoomV3Pdu {
        room_id: room_id!("!n8f893n9:example.com").to_owned(),
        sender: user_id!("@sender:example.com").to_owned(),
        origin: "matrix.org".into(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(1_592_050_773_658_u64.try_into().unwrap()),
        kind: EventType::RoomPowerLevels,
        content: to_raw_json_value(&json!({ "testing": 123 })).unwrap(),
        state_key: Some("state".into()),
        prev_events: vec![event_id!("$previousevent:matrix.org").to_owned()],
        depth: 2_u32.into(),
        auth_events: vec![event_id!("$someauthevent:matrix.org").to_owned()],
        redacts: Some(event_id!("$9654:matrix.org").to_owned()),
        unsigned,
        hashes: EventHash::new("1233543bABACDEF".into()),
        signatures,
    };
    let pdu_stub = Pdu::RoomV3Pdu(v3_pdu);
    let json = json!({
        "room_id": "!n8f893n9:example.com",
        "sender": "@sender:example.com",
        "origin": "matrix.org",
        "origin_server_ts": 1_592_050_773_658_u64,
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
fn deserialize_pdu_as_v1() {
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
        #[cfg(not(feature = "unstable-exhaustive-types"))]
        _ => unreachable!("new PDU version"),
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
            assert_eq!(v3_pdu.auth_events.first().unwrap(), event_id!("$abc123:matrix.org"));
        }
        #[cfg(not(feature = "unstable-exhaustive-types"))]
        _ => unreachable!("new PDU version"),
    }
}
