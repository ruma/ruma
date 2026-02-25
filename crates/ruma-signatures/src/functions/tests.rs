use std::collections::BTreeMap;

use assert_matches2::assert_matches;
use ruma_common::{
    CanonicalJsonValue, ServerSigningKeyId, SigningKeyAlgorithm,
    room_version_rules::{RoomVersionRules, SignaturesRules},
    serde::Base64,
    server_name,
};
use serde_json::json;

use super::{
    canonical_json, servers_to_check_signatures, sign_json, verify_canonical_json_bytes,
    verify_event,
};
use crate::{
    Ed25519KeyPair, Error, KeyPair, PublicKeyMap, PublicKeySet, VerificationError, Verified,
};

fn generate_key_pair(name: &str) -> Ed25519KeyPair {
    let key_content = Ed25519KeyPair::generate().unwrap();
    Ed25519KeyPair::from_der(&key_content, name.to_owned())
        .unwrap_or_else(|_| panic!("{:?}", &key_content))
}

fn add_key_to_map(public_key_map: &mut PublicKeyMap, name: &str, pair: &Ed25519KeyPair) {
    let sender_key_map = public_key_map.entry(name.to_owned()).or_default();
    let encoded_public_key = Base64::new(pair.public_key().to_vec());
    let version = ServerSigningKeyId::from_parts(
        SigningKeyAlgorithm::Ed25519,
        &pair.version().try_into().unwrap(),
    );

    sender_key_map.insert(version.to_string(), encoded_public_key);
}

fn add_invalid_key_to_map(public_key_map: &mut PublicKeyMap, name: &str, pair: &Ed25519KeyPair) {
    let sender_key_map = public_key_map.entry(name.to_owned()).or_default();
    let encoded_public_key = Base64::new(pair.public_key().to_vec());
    let version = ServerSigningKeyId::from_parts(
        SigningKeyAlgorithm::from("an-unknown-algorithm"),
        &pair.version().try_into().unwrap(),
    );

    sender_key_map.insert(version.to_string(), encoded_public_key);
}

#[test]
fn canonical_json_complex() {
    let data = json!({
        "auth": {
            "success": true,
            "mxid": "@john.doe:example.com",
            "profile": {
                "display_name": "John Doe",
                "three_pids": [
                    {
                        "medium": "email",
                        "address": "john.doe@example.org"
                    },
                    {
                        "medium": "msisdn",
                        "address": "123456789"
                    }
                ]
            }
        }
    });

    let canonical = r#"{"auth":{"mxid":"@john.doe:example.com","profile":{"display_name":"John Doe","three_pids":[{"address":"john.doe@example.org","medium":"email"},{"address":"123456789","medium":"msisdn"}]},"success":true}}"#;

    let CanonicalJsonValue::Object(object) = CanonicalJsonValue::try_from(data).unwrap() else {
        unreachable!();
    };

    assert_eq!(canonical_json(&object).unwrap(), canonical);
}

#[test]
fn verify_event_does_not_check_signatures_invite_via_third_party_id() {
    let signed_event = serde_json::from_str(
            r#"{
                "auth_events": [],
                "content": {
                    "membership": "invite",
                    "third_party_invite": {
                        "display_name": "alice",
                        "signed": {
                            "mxid": "@alice:example.org",
                            "signatures": {
                                "magic.forest": {
                                    "ed25519:3": "fQpGIW1Snz+pwLZu6sTy2aHy/DYWWTspTJRPyNp0PKkymfIsNffysMl6ObMMFdIJhk6g6pwlIqZ54rxo8SLmAg"
                                }
                            },
                            "token": "abc123"
                        }
                    }
                },
                "depth": 3,
                "hashes": {
                    "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
                },
                "origin": "domain",
                "origin_server_ts": 1000000,
                "prev_events": [],
                "room_id": "!x:domain",
                "sender": "@a:domain",
                "signatures": {
                    "domain": {
                        "ed25519:1": "KxwGjPSDEtvnFgU00fwFz+l6d2pJM6XBIaMEn81SXPTRl16AqLAYqfIReFGZlHi5KLjAWbOoMszkwsQma+lYAg"
                    }
                },
                "type": "m.room.member",
                "unsigned": {
                    "age_ts": 1000000
                }
            }"#
        ).unwrap();

    let public_key_map = BTreeMap::new();
    let verification = verify_event(&public_key_map, &signed_event, &RoomVersionRules::V6).unwrap();

    assert_eq!(verification, Verified::Signatures);
}

#[test]
fn verify_event_check_signatures_for_both_sender_and_event_id() {
    let key_pair_sender = generate_key_pair("1");
    let key_pair_event = generate_key_pair("2");
    let mut signed_event = serde_json::from_str(
        r#"{
                "event_id": "$event_id:domain-event",
                "auth_events": [],
                "content": {},
                "depth": 3,
                "hashes": {
                    "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
                },
                "origin": "domain",
                "origin_server_ts": 1000000,
                "prev_events": [],
                "room_id": "!x:domain",
                "sender": "@name:domain-sender",
                "type": "X",
                "unsigned": {
                    "age_ts": 1000000
                }
            }"#,
    )
    .unwrap();
    sign_json("domain-sender", &key_pair_sender, &mut signed_event).unwrap();
    sign_json("domain-event", &key_pair_event, &mut signed_event).unwrap();

    let mut public_key_map = BTreeMap::new();
    add_key_to_map(&mut public_key_map, "domain-sender", &key_pair_sender);
    add_key_to_map(&mut public_key_map, "domain-event", &key_pair_event);

    let verification = verify_event(&public_key_map, &signed_event, &RoomVersionRules::V1).unwrap();

    assert_eq!(verification, Verified::Signatures);
}

#[test]
fn verify_event_check_signatures_for_authorized_user() {
    let key_pair_sender = generate_key_pair("1");
    let key_pair_authorized = generate_key_pair("2");
    let mut signed_event = serde_json::from_str(
        r#"{
                "event_id": "$event_id:domain-event",
                "auth_events": [],
                "content": {
                    "membership": "join",
                    "join_authorised_via_users_server": "@authorized:domain-authorized"
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
    sign_json("domain-sender", &key_pair_sender, &mut signed_event).unwrap();
    sign_json("domain-authorized", &key_pair_authorized, &mut signed_event).unwrap();

    let mut public_key_map = BTreeMap::new();
    add_key_to_map(&mut public_key_map, "domain-sender", &key_pair_sender);
    add_key_to_map(&mut public_key_map, "domain-authorized", &key_pair_authorized);

    let verification = verify_event(&public_key_map, &signed_event, &RoomVersionRules::V9).unwrap();

    assert_eq!(verification, Verified::Signatures);
}

#[test]
fn verification_fails_if_missing_signatures_for_authorized_user() {
    let key_pair_sender = generate_key_pair("1");
    let mut signed_event = serde_json::from_str(
        r#"{
                "event_id": "$event_id:domain-event",
                "auth_events": [],
                "content": {"join_authorised_via_users_server": "@authorized:domain-authorized"},
                "depth": 3,
                "hashes": {
                    "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
                },
                "origin": "domain",
                "origin_server_ts": 1000000,
                "prev_events": [],
                "room_id": "!x:domain",
                "sender": "@name:domain-sender",
                "type": "X",
                "unsigned": {
                    "age_ts": 1000000
                }
            }"#,
    )
    .unwrap();
    sign_json("domain-sender", &key_pair_sender, &mut signed_event).unwrap();

    let mut public_key_map = BTreeMap::new();
    add_key_to_map(&mut public_key_map, "domain-sender", &key_pair_sender);

    let verification_result = verify_event(&public_key_map, &signed_event, &RoomVersionRules::V9);

    assert_matches!(
        verification_result,
        Err(Error::Verification(VerificationError::NoSignaturesForEntity(server)))
    );
    assert_eq!(server, "domain-authorized");
}

#[test]
fn verification_fails_if_required_keys_are_not_given() {
    let key_pair_sender = generate_key_pair("1");

    let mut signed_event = serde_json::from_str(
        r#"{
                "auth_events": [],
                "content": {},
                "depth": 3,
                "hashes": {
                    "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
                },
                "origin": "domain",
                "origin_server_ts": 1000000,
                "prev_events": [],
                "room_id": "!x:domain",
                "sender": "@name:domain-sender",
                "type": "X",
                "unsigned": {
                    "age_ts": 1000000
                }
            }"#,
    )
    .unwrap();
    sign_json("domain-sender", &key_pair_sender, &mut signed_event).unwrap();

    // Verify with an empty public key map should fail due to missing public keys
    let public_key_map = BTreeMap::new();
    let verification_result = verify_event(&public_key_map, &signed_event, &RoomVersionRules::V6);

    assert_matches!(
        verification_result,
        Err(Error::Verification(VerificationError::NoPublicKeysForEntity(entity)))
    );
    assert_eq!(entity, "domain-sender");
}

#[test]
fn verify_event_fails_if_public_key_is_invalid() {
    let key_pair_sender = generate_key_pair("1");

    let mut signed_event = serde_json::from_str(
        r#"{
                "auth_events": [],
                "content": {},
                "depth": 3,
                "hashes": {
                    "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
                },
                "origin": "domain",
                "origin_server_ts": 1000000,
                "prev_events": [],
                "room_id": "!x:domain",
                "sender": "@name:domain-sender",
                "type": "X",
                "unsigned": {
                    "age_ts": 1000000
                }
            }"#,
    )
    .unwrap();
    sign_json("domain-sender", &key_pair_sender, &mut signed_event).unwrap();

    let mut public_key_map = PublicKeyMap::new();
    let mut sender_key_map = PublicKeySet::new();
    let newly_generated_key_pair = generate_key_pair("2");
    let encoded_public_key = Base64::new(newly_generated_key_pair.public_key().to_vec());
    let version = ServerSigningKeyId::from_parts(
        SigningKeyAlgorithm::Ed25519,
        &key_pair_sender.version().try_into().unwrap(),
    );
    sender_key_map.insert(version.to_string(), encoded_public_key);
    public_key_map.insert("domain-sender".to_owned(), sender_key_map);

    let verification_result = verify_event(&public_key_map, &signed_event, &RoomVersionRules::V6);

    assert_matches!(
        verification_result,
        Err(Error::Verification(VerificationError::Signature(error)))
    );
    // dalek doesn't expose InternalError :(
    // https://github.com/dalek-cryptography/ed25519-dalek/issues/174
    assert!(format!("{error:?}").contains("Some(Verification equation was not satisfied)"));
}

#[test]
fn verify_event_check_signatures_for_sender_is_allowed_with_unknown_algorithms_in_key_map() {
    let key_pair_sender = generate_key_pair("1");
    let mut signed_event = serde_json::from_str(
        r#"{
                "auth_events": [],
                "content": {},
                "depth": 3,
                "hashes": {
                    "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
                },
                "origin": "domain",
                "origin_server_ts": 1000000,
                "prev_events": [],
                "room_id": "!x:domain",
                "sender": "@name:domain-sender",
                "type": "X",
                "unsigned": {
                    "age_ts": 1000000
                }
            }"#,
    )
    .unwrap();
    sign_json("domain-sender", &key_pair_sender, &mut signed_event).unwrap();

    let mut public_key_map = BTreeMap::new();
    add_key_to_map(&mut public_key_map, "domain-sender", &key_pair_sender);
    add_invalid_key_to_map(&mut public_key_map, "domain-sender", &generate_key_pair("2"));

    let verification = verify_event(&public_key_map, &signed_event, &RoomVersionRules::V6).unwrap();

    assert_eq!(verification, Verified::Signatures);
}

#[test]
fn verify_event_fails_with_missing_key_when_event_is_signed_multiple_times_by_same_entity() {
    let key_pair_sender = generate_key_pair("1");
    let secondary_key_pair_sender = generate_key_pair("2");
    let mut signed_event = serde_json::from_str(
        r#"{
                "auth_events": [],
                "content": {},
                "depth": 3,
                "hashes": {
                    "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
                },
                "origin": "domain",
                "origin_server_ts": 1000000,
                "prev_events": [],
                "room_id": "!x:domain",
                "sender": "@name:domain-sender",
                "type": "X",
                "unsigned": {
                    "age_ts": 1000000
                }
            }"#,
    )
    .unwrap();
    sign_json("domain-sender", &key_pair_sender, &mut signed_event).unwrap();
    sign_json("domain-sender", &secondary_key_pair_sender, &mut signed_event).unwrap();

    let mut public_key_map = BTreeMap::new();
    add_key_to_map(&mut public_key_map, "domain-sender", &key_pair_sender);

    let verification_result = verify_event(&public_key_map, &signed_event, &RoomVersionRules::V6);

    assert_matches!(
        verification_result,
        Err(Error::Verification(VerificationError::PublicKeyNotFound { entity, key_id }))
    );
    assert_eq!(entity, "domain-sender");
    assert_eq!(key_id, "ed25519:2");
}

#[test]
fn verify_event_checks_all_signatures_from_sender_entity() {
    let key_pair_sender = generate_key_pair("1");
    let secondary_key_pair_sender = generate_key_pair("2");
    let mut signed_event = serde_json::from_str(
        r#"{
                "auth_events": [],
                "content": {},
                "depth": 3,
                "hashes": {
                    "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
                },
                "origin": "domain",
                "origin_server_ts": 1000000,
                "prev_events": [],
                "room_id": "!x:domain",
                "sender": "@name:domain-sender",
                "type": "X",
                "unsigned": {
                    "age_ts": 1000000
                }
            }"#,
    )
    .unwrap();
    sign_json("domain-sender", &key_pair_sender, &mut signed_event).unwrap();
    sign_json("domain-sender", &secondary_key_pair_sender, &mut signed_event).unwrap();

    let mut public_key_map = BTreeMap::new();
    add_key_to_map(&mut public_key_map, "domain-sender", &key_pair_sender);
    add_key_to_map(&mut public_key_map, "domain-sender", &secondary_key_pair_sender);

    let verification = verify_event(&public_key_map, &signed_event, &RoomVersionRules::V6).unwrap();

    assert_eq!(verification, Verified::Signatures);
}

#[test]
fn verify_event_with_single_key_with_unknown_algorithm_should_not_accept_event() {
    let key_pair_sender = generate_key_pair("1");
    let signed_event = serde_json::from_str(
            r#"{
                "auth_events": [],
                "content": {},
                "depth": 3,
                "hashes": {
                    "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
                },
                "origin": "domain",
                "origin_server_ts": 1000000,
                "prev_events": [],
                "room_id": "!x:domain",
                "sender": "@name:domain-sender",
                "type": "X",
                "unsigned": {
                    "age_ts": 1000000
                },
                "signatures": {
                    "domain-sender": {
                        "an-unknown-algorithm:1": "pE5UT/4JiY7YZDtZDOsEaxc0wblurdoYqNQx4bCXORA3vLFOGOK10Q/xXVLPWWgIKo15LNvWwWd/2YjmdPvYCg"
                    }
                }
            }"#,
        )
        .unwrap();

    let mut public_key_map = BTreeMap::new();
    add_invalid_key_to_map(&mut public_key_map, "domain-sender", &key_pair_sender);

    let verification_result = verify_event(&public_key_map, &signed_event, &RoomVersionRules::V6);
    assert_matches!(
        verification_result,
        Err(Error::Verification(VerificationError::NoSupportedSignatureForEntity(entity)))
    );
    assert_eq!(entity, "domain-sender");
}

#[test]
fn servers_to_check_signatures_message() {
    let message_event_json = json!({
        "event_id": "$event_id:domain-event",
        "auth_events": [
            "$room_create",
            "$power_levels",
            "$sender_room_member",
        ],
        "content": {
            "msgtype": "text",
            "body": "Hello world!",
        },
        "depth": 3,
        "hashes": {
            "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
        },
        "origin": "domain",
        "origin_server_ts": 1_000_000,
        "prev_events": [
            "$another_message",
        ],
        "room_id": "!x:domain",
        "sender": "@name:domain-sender",
        "type": "m.room.message",
        "unsigned": {
            "age_ts": 1_000_000,
        }
    });
    let object = serde_json::from_value(message_event_json).unwrap();

    // Check for room v1.
    let servers = servers_to_check_signatures(&object, &SignaturesRules::V1).unwrap();
    assert_eq!(servers.len(), 2);
    assert!(servers.contains(&server_name!("domain-sender")));
    assert!(servers.contains(&server_name!("domain-event")));

    // Check for room v3.
    let servers = servers_to_check_signatures(&object, &SignaturesRules::V3).unwrap();
    assert_eq!(servers.len(), 1);
    assert!(servers.contains(&server_name!("domain-sender")));
}

#[test]
fn servers_to_check_signatures_invite_via_third_party() {
    let message_event_json = json!({
        "event_id": "$event_id:domain-event",
        "auth_events": [
            "$room_create",
            "$power_levels",
            "$sender_room_member",
        ],
        "content": {
            "membership": "invite",
            "third_party_invite": {},
        },
        "depth": 3,
        "hashes": {
            "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
        },
        "origin": "domain",
        "origin_server_ts": 1_000_000,
        "prev_events": [
            "$another_message",
        ],
        "room_id": "!x:domain",
        "sender": "@name:domain-sender",
        "state_key": "@name:domain-target-user",
        "type": "m.room.member",
        "unsigned": {
            "age_ts": 1_000_000,
        }
    });
    let object = serde_json::from_value(message_event_json).unwrap();

    // Check for room v1.
    let servers = servers_to_check_signatures(&object, &SignaturesRules::V1).unwrap();
    assert_eq!(servers.len(), 1);
    assert!(servers.contains(&server_name!("domain-event")));

    // Check for room v3.
    let servers = servers_to_check_signatures(&object, &SignaturesRules::V3).unwrap();
    assert_eq!(servers.len(), 0);
}

#[test]
fn servers_to_check_signatures_restricted() {
    let message_event_json = json!({
        "event_id": "$event_id:domain-event",
        "auth_events": [
            "$room_create",
            "$power_levels",
            "$sender_room_member",
        ],
        "content": {
            "membership": "join",
            "join_authorised_via_users_server": "@name:domain-authorize-user",
        },
        "depth": 3,
        "hashes": {
            "sha256": "5jM4wQpv6lnBo7CLIghJuHdW+s2CMBJPUOGOC89ncos"
        },
        "origin": "domain",
        "origin_server_ts": 1_000_000,
        "prev_events": [
            "$another_message",
        ],
        "room_id": "!x:domain",
        "sender": "@name:domain-sender",
        "state_key": "@name:domain-sender",
        "type": "m.room.member",
        "unsigned": {
            "age_ts": 1_000_000,
        }
    });
    let object = serde_json::from_value(message_event_json).unwrap();

    // Check for room v1.
    let servers = servers_to_check_signatures(&object, &SignaturesRules::V1).unwrap();
    assert_eq!(servers.len(), 2);
    assert!(servers.contains(&server_name!("domain-sender")));
    assert!(servers.contains(&server_name!("domain-event")));

    // Check for room v3.
    let servers = servers_to_check_signatures(&object, &SignaturesRules::V3).unwrap();
    assert_eq!(servers.len(), 1);
    assert!(servers.contains(&server_name!("domain-sender")));

    // Check for room v8.
    let servers = servers_to_check_signatures(&object, &SignaturesRules::V8).unwrap();
    assert_eq!(servers.len(), 2);
    assert!(servers.contains(&server_name!("domain-sender")));
    assert!(servers.contains(&server_name!("domain-authorize-user")));
}

#[test]
fn verify_canonical_json_bytes_success() {
    let json = serde_json::from_value(json!({
        "foo": "bar",
        "bat": "baz",
    }))
    .unwrap();
    let canonical_json = canonical_json(&json).unwrap();

    let key_pair = generate_key_pair("1");
    let signature = key_pair.sign(canonical_json.as_bytes());

    verify_canonical_json_bytes(
        &signature.algorithm(),
        key_pair.public_key().as_slice(),
        signature.as_bytes(),
        canonical_json.as_bytes(),
    )
    .unwrap();
}

#[test]
fn verify_canonical_json_bytes_unsupported_algorithm() {
    let json = serde_json::from_value(json!({
        "foo": "bar",
        "bat": "baz",
    }))
    .unwrap();
    let canonical_json = canonical_json(&json).unwrap();

    let key_pair = generate_key_pair("1");
    let signature = key_pair.sign(canonical_json.as_bytes());

    let err = verify_canonical_json_bytes(
        &"unknown".into(),
        key_pair.public_key().as_slice(),
        signature.as_bytes(),
        canonical_json.as_bytes(),
    )
    .unwrap_err();
    assert_matches!(err, Error::Verification(VerificationError::UnsupportedAlgorithm));
}

#[test]
fn verify_canonical_json_bytes_wrong_key() {
    let json = serde_json::from_value(json!({
        "foo": "bar",
        "bat": "baz",
    }))
    .unwrap();
    let canonical_json = canonical_json(&json).unwrap();

    let valid_key_pair = generate_key_pair("1");
    let signature = valid_key_pair.sign(canonical_json.as_bytes());

    let wrong_key_pair = generate_key_pair("2");

    let err = verify_canonical_json_bytes(
        &"ed25519".into(),
        wrong_key_pair.public_key().as_slice(),
        signature.as_bytes(),
        canonical_json.as_bytes(),
    )
    .unwrap_err();
    assert_matches!(err, Error::Verification(VerificationError::Signature(_)));
}
