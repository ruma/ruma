use std::collections::BTreeMap;

use ruma_common::{serde::Base64, RoomVersionId, ServerSigningKeyId, SigningKeyAlgorithm};
use ruma_signatures::{sign_json, verify_event, Ed25519KeyPair, PublicKeyMap, Verified};

static PKCS8_ED25519_DER: &[u8] = include_bytes!("./keys/ed25519.der");

fn add_key_to_map(public_key_map: &mut PublicKeyMap, name: &str, pair: &Ed25519KeyPair) {
    let sender_key_map = public_key_map.entry(name.to_owned()).or_default();
    let encoded_public_key = Base64::new(pair.public_key().to_vec());
    let version = ServerSigningKeyId::from_parts(
        SigningKeyAlgorithm::Ed25519,
        pair.version().try_into().unwrap(),
    );

    sender_key_map.insert(version.to_string(), encoded_public_key);
}

#[test]
fn verify_event_check_signatures_for_authorized_user() {
    let keypair = Ed25519KeyPair::from_der(PKCS8_ED25519_DER, "1".to_owned()).unwrap();

    let mut signed_event = serde_json::from_str(
        r#"{
            "event_id": "$event_id:domain-event",
            "auth_events": [],
            "content": {
                "membership": "join"
            },
            "depth": 3,
            "hashes": {
                "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
            },
            "origin": "domain",
            "origin_server_ts": 1000000,
            "prev_events": [],
            "room_id": "!x:domain",
            "sender": "@name:domain-sender",
            "type": "m.room.member",
            "unsigned": {
                "age_ts": 1000000
            }
        }"#,
    )
    .unwrap();
    sign_json("domain-sender", &keypair, &mut signed_event).unwrap();

    let mut public_key_map = BTreeMap::new();
    add_key_to_map(&mut public_key_map, "domain-sender", &keypair);

    let verification = verify_event(&public_key_map, &signed_event, &RoomVersionId::V9).unwrap();

    assert_eq!(verification, Verified::Signatures);

    let signatures = signed_event.get("signatures").unwrap().as_object().unwrap();
    let domain_sender_signatures = signatures.get("domain-sender").unwrap().as_object().unwrap();
    let signature = domain_sender_signatures.get("ed25519:1").unwrap().as_str().unwrap();
    insta::assert_snapshot!(signature);
}
