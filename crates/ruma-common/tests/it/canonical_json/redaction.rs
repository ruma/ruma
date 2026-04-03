use assert_matches2::assert_let;
use ruma_common::{
    canonical_json::{
        CanonicalJsonValue, RedactingSerializer, assert_to_canonical_json_eq, redact,
    },
    room_version_rules::RedactionRules,
};
use serde_json::{json, to_string as to_json_string};

#[test]
fn redact_power_levels() {
    let original_event = json!({
        "content": {
            "ban": 50,
            "events": {
                "m.room.avatar": 50,
                "m.room.canonical_alias": 50,
                "m.room.history_visibility": 100,
                "m.room.name": 50,
                "m.room.power_levels": 100
            },
            "events_default": 0,
            "invite": 0,
            "kick": 50,
            "redact": 50,
            "state_default": 50,
            "users": {
                "@example:localhost": 100
            },
            "users_default": 0
        },
        "event_id": "$15139375512JaHAW:localhost",
        "origin_server_ts": 45,
        "sender": "@example:localhost",
        "room_id": "!room:localhost",
        "state_key": "",
        "type": "m.room.power_levels",
        "unsigned": {
            "age": 45
        }
    });
    assert_let!(
        Ok(CanonicalJsonValue::Object(original_object)) =
            CanonicalJsonValue::try_from(original_event)
    );

    // In room v1, most of the content keys are kept except `invite`.
    let expected = json!({
        "content": {
            "ban": 50,
            "events": {
                "m.room.avatar": 50,
                "m.room.canonical_alias": 50,
                "m.room.history_visibility": 100,
                "m.room.name": 50,
                "m.room.power_levels": 100
            },
            "events_default": 0,
            "kick": 50,
            "redact": 50,
            "state_default": 50,
            "users": {
                "@example:localhost": 100
            },
            "users_default": 0
        },
        "event_id": "$15139375512JaHAW:localhost",
        "origin_server_ts": 45,
        "sender": "@example:localhost",
        "room_id": "!room:localhost",
        "state_key": "",
        "type": "m.room.power_levels",
    });
    let redacted = redact(original_object.clone(), &RedactionRules::V1, None).unwrap();
    assert_to_canonical_json_eq!(redacted, expected.clone());
    let redacted_canonical =
        RedactingSerializer::new().rules(&RedactionRules::V1).serialize(&original_object).unwrap();
    assert_eq!(redacted_canonical, to_json_string(&expected).unwrap());

    // In room v11, the `invite` key is also kept.
    let expected = json!({
        "content": {
            "ban": 50,
            "events": {
                "m.room.avatar": 50,
                "m.room.canonical_alias": 50,
                "m.room.history_visibility": 100,
                "m.room.name": 50,
                "m.room.power_levels": 100
            },
            "events_default": 0,
            "invite": 0,
            "kick": 50,
            "redact": 50,
            "state_default": 50,
            "users": {
                "@example:localhost": 100
            },
            "users_default": 0
        },
        "event_id": "$15139375512JaHAW:localhost",
        "origin_server_ts": 45,
        "sender": "@example:localhost",
        "room_id": "!room:localhost",
        "state_key": "",
        "type": "m.room.power_levels",
    });
    let redacted = redact(original_object.clone(), &RedactionRules::V11, None).unwrap();
    assert_to_canonical_json_eq!(redacted, expected.clone());
    let redacted_canonical =
        RedactingSerializer::new().rules(&RedactionRules::V11).serialize(&original_object).unwrap();
    assert_eq!(redacted_canonical, to_json_string(&expected).unwrap());
}

#[test]
fn redact_room_aliases() {
    let original_event = json!({
        "content": {
            "aliases": ["#somewhere:localhost"]
        },
        "event_id": "$152037280074GZeOm:localhost",
        "origin_server_ts": 1,
        "sender": "@example:localhost",
        "state_key": "room.com",
        "room_id": "!room:room.com",
        "type": "m.room.aliases",
        "unsigned": {
            "age": 1
        }
    });
    assert_let!(
        Ok(CanonicalJsonValue::Object(original_object)) =
            CanonicalJsonValue::try_from(original_event)
    );

    // In room v1, the aliases are kept.
    let expected = json!({
        "content": {
            "aliases": ["#somewhere:localhost"]
        },
        "event_id": "$152037280074GZeOm:localhost",
        "origin_server_ts": 1,
        "sender": "@example:localhost",
        "state_key": "room.com",
        "room_id": "!room:room.com",
        "type": "m.room.aliases",
    });
    let redacted = redact(original_object.clone(), &RedactionRules::V1, None).unwrap();
    assert_to_canonical_json_eq!(redacted, expected.clone());
    let redacted_canonical =
        RedactingSerializer::new().rules(&RedactionRules::V1).serialize(&original_object).unwrap();
    assert_eq!(redacted_canonical, to_json_string(&expected).unwrap());

    // In room v11, all the content keys are removed.
    let expected = json!({
        "content": {},
        "event_id": "$152037280074GZeOm:localhost",
        "origin_server_ts": 1,
        "sender": "@example:localhost",
        "state_key": "room.com",
        "room_id": "!room:room.com",
        "type": "m.room.aliases",
    });
    let redacted = redact(original_object.clone(), &RedactionRules::V9, None).unwrap();
    assert_to_canonical_json_eq!(redacted, expected.clone());
    let redacted_canonical =
        RedactingSerializer::new().rules(&RedactionRules::V9).serialize(&original_object).unwrap();
    assert_eq!(redacted_canonical, to_json_string(&expected).unwrap());
}

#[test]
fn redact_room_create() {
    let original_event = json!({
        "content": {
            "creator": "@example:example.org",
            "m.federate": true,
            "predecessor": {
                "event_id": "$something",
                "room_id": "!oldroom:example.org"
            },
            "room_version": "11",
            "local.custom.key": "foo",
        },
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 1_432_735,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "state_key": "",
        "type": "m.room.create",
        "unsigned": {
          "age": 1234,
        },
    });
    assert_let!(
        Ok(CanonicalJsonValue::Object(original_object)) =
            CanonicalJsonValue::try_from(original_event)
    );

    // In room v1, only the creator is kept.
    let expected = json!({
        "content": {
            "creator": "@example:example.org",
        },
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 1_432_735,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "state_key": "",
        "type": "m.room.create",
    });
    let redacted = redact(original_object.clone(), &RedactionRules::V1, None).unwrap();
    assert_to_canonical_json_eq!(redacted, expected.clone());
    let redacted_canonical =
        RedactingSerializer::new().rules(&RedactionRules::V1).serialize(&original_object).unwrap();
    assert_eq!(redacted_canonical, to_json_string(&expected).unwrap());

    // In room v11, all the content keys are kept.
    let expected = json!({
        "content": {
            "creator": "@example:example.org",
            "m.federate": true,
            "predecessor": {
                "event_id": "$something",
                "room_id": "!oldroom:example.org"
            },
            "room_version": "11",
            "local.custom.key": "foo",
        },
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 1_432_735,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "state_key": "",
        "type": "m.room.create",
    });
    let redacted = redact(original_object.clone(), &RedactionRules::V11, None).unwrap();
    assert_to_canonical_json_eq!(redacted, expected.clone());
    let redacted_canonical =
        RedactingSerializer::new().rules(&RedactionRules::V11).serialize(&original_object).unwrap();
    assert_eq!(redacted_canonical, to_json_string(&expected).unwrap());
}

#[test]
fn redact_room_member_third_party_invite() {
    let original_event = json!({
        "content": {
            "membership": "invite",
            "displayname": "Example",
            "third_party_invite": {
                "display_name": "example",
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
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 1_432_735,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "state_key": "@alice:example.org",
        "type": "m.room.member",
        "unsigned": {
          "age": 1234,
        },
    });
    assert_let!(
        Ok(CanonicalJsonValue::Object(original_object)) =
            CanonicalJsonValue::try_from(original_event)
    );

    // In room v1, only `membership` is kept.
    let expected = json!({
        "content": {
            "membership": "invite",
        },
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 1_432_735,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "state_key": "@alice:example.org",
        "type": "m.room.member",
    });
    let redacted = redact(original_object.clone(), &RedactionRules::V1, None).unwrap();
    assert_to_canonical_json_eq!(redacted, expected.clone());
    let redacted_canonical =
        RedactingSerializer::new().rules(&RedactionRules::V1).serialize(&original_object).unwrap();
    assert_eq!(redacted_canonical, to_json_string(&expected).unwrap());

    // In room v11, `membership` and `third_party_invite.signed` are kept.
    let expected = json!({
        "content": {
            "membership": "invite",
            "third_party_invite": {
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
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 1_432_735,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "state_key": "@alice:example.org",
        "type": "m.room.member",
    });
    let redacted = redact(original_object.clone(), &RedactionRules::V11, None).unwrap();
    assert_to_canonical_json_eq!(redacted, expected.clone());
    let redacted_canonical =
        RedactingSerializer::new().rules(&RedactionRules::V11).serialize(&original_object).unwrap();
    assert_eq!(redacted_canonical, to_json_string(&expected).unwrap());
}

#[test]
fn redact_room_member_join_authorised_via_users_server() {
    let original_event = json!({
        "content": {
            "membership": "join",
            "displayname": "Example",
            "join_authorised_via_users_server": "@alice:example.org",
        },
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 1_432_735,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "state_key": "@example:example.org",
        "type": "m.room.member",
        "unsigned": {
          "age": 1234,
        },
    });
    assert_let!(
        Ok(CanonicalJsonValue::Object(original_object)) =
            CanonicalJsonValue::try_from(original_event)
    );

    // In room v1, only `membership` is kept.
    let expected = json!({
        "content": {
            "membership": "join",
        },
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 1_432_735,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "state_key": "@example:example.org",
        "type": "m.room.member",
    });
    let redacted = redact(original_object.clone(), &RedactionRules::V1, None).unwrap();
    assert_to_canonical_json_eq!(redacted, expected.clone());
    let redacted_canonical =
        RedactingSerializer::new().rules(&RedactionRules::V1).serialize(&original_object).unwrap();
    assert_eq!(redacted_canonical, to_json_string(&expected).unwrap());

    // In room v11, `membership` and `join_authorised_via_users_server` are kept.
    let expected = json!({
        "content": {
            "membership": "join",
            "join_authorised_via_users_server": "@alice:example.org",
        },
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 1_432_735,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "state_key": "@example:example.org",
        "type": "m.room.member",
    });
    let redacted = redact(original_object.clone(), &RedactionRules::V11, None).unwrap();
    assert_to_canonical_json_eq!(redacted, expected.clone());
    let redacted_canonical =
        RedactingSerializer::new().rules(&RedactionRules::V11).serialize(&original_object).unwrap();
    assert_eq!(redacted_canonical, to_json_string(&expected).unwrap());
}

#[test]
fn redact_room_join_rules() {
    let original_event = json!({
        "content": {
            "join_rule": "restricted",
            "allow": [
                {
                    "room_id": "!other:example.org",
                    "type": "m.room_membership",
                },
            ],
        },
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 1_432_735,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "state_key": "",
        "type": "m.room.join_rules",
    });
    assert_let!(
        Ok(CanonicalJsonValue::Object(original_object)) =
            CanonicalJsonValue::try_from(original_event)
    );

    // In v1, only `join_rule` is kept.
    let expected = json!({
        "content": {
            "join_rule": "restricted",
        },
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 1_432_735,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "state_key": "",
        "type": "m.room.join_rules",
    });
    let redacted = redact(original_object.clone(), &RedactionRules::V1, None).unwrap();
    assert_to_canonical_json_eq!(redacted, expected.clone());
    let redacted_canonical =
        RedactingSerializer::new().rules(&RedactionRules::V1).serialize(&original_object).unwrap();
    assert_eq!(redacted_canonical, to_json_string(&expected).unwrap());

    // In v11, `join_rule` and `allow` are kept.
    let expected = json!({
        "content": {
            "join_rule": "restricted",
            "allow": [
                {
                    "room_id": "!other:example.org",
                    "type": "m.room_membership",
                },
            ],
        },
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 1_432_735,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "state_key": "",
        "type": "m.room.join_rules",
    });
    let redacted = redact(original_object.clone(), &RedactionRules::V11, None).unwrap();
    assert_to_canonical_json_eq!(redacted, expected.clone());
    let redacted_canonical =
        RedactingSerializer::new().rules(&RedactionRules::V11).serialize(&original_object).unwrap();
    assert_eq!(redacted_canonical, to_json_string(&expected).unwrap());
}

#[test]
fn redact_room_history_visibility() {
    let original_event = json!({
        "content": {
            "history_visibility": "invited",
        },
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 1_432_735,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "state_key": "",
        "type": "m.room.history_visibility",
    });
    assert_let!(
        Ok(CanonicalJsonValue::Object(original_object)) =
            CanonicalJsonValue::try_from(original_event)
    );

    // `history_visibility` is kept.
    let expected = json!({
        "content": {
            "history_visibility": "invited",
        },
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 1_432_735,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "state_key": "",
        "type": "m.room.history_visibility",
    });
    let redacted = redact(original_object.clone(), &RedactionRules::V1, None).unwrap();
    assert_to_canonical_json_eq!(redacted, expected.clone());
    let redacted_canonical =
        RedactingSerializer::new().rules(&RedactionRules::V1).serialize(&original_object).unwrap();
    assert_eq!(redacted_canonical, to_json_string(&expected).unwrap());
}

#[test]
fn redact_room_redaction() {
    let original_event = json!({
        "content": {
            "reason": "Spamming",
            "redacts": "$fukweghifu23",
        },
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 1_432_735,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "type": "m.room.redaction",
        "redacts": "$fukweghifu23",
    });
    assert_let!(
        Ok(CanonicalJsonValue::Object(original_object)) =
            CanonicalJsonValue::try_from(original_event)
    );

    // In v1, all content keys are redacted.
    let expected = json!({
        "content": {},
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 1_432_735,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "type": "m.room.redaction",
    });
    let redacted = redact(original_object.clone(), &RedactionRules::V1, None).unwrap();
    assert_to_canonical_json_eq!(redacted, expected.clone(),);
    let redacted_canonical =
        RedactingSerializer::new().rules(&RedactionRules::V1).serialize(&original_object).unwrap();
    assert_eq!(redacted_canonical, to_json_string(&expected).unwrap());

    // In v11, the `redacts` key is kept.
    let expected = json!({
        "content": {
            "redacts": "$fukweghifu23",
        },
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 1_432_735,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "type": "m.room.redaction",
    });
    let redacted = redact(original_object.clone(), &RedactionRules::V11, None).unwrap();
    assert_to_canonical_json_eq!(redacted, expected.clone(),);
    let redacted_canonical =
        RedactingSerializer::new().rules(&RedactionRules::V11).serialize(&original_object).unwrap();
    assert_eq!(redacted_canonical, to_json_string(&expected).unwrap());
}

#[test]
fn redacting_serializer_custom_redacted_root_fields() {
    let original_event = json!({
        "content": {
            "foo": "bar",
        },
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 1_432_735,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "type": "local.custom.type",
        "hashes": {
            "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        },
        "signatures": {
            "example.org": {
                "ed25519:10": "cd372fb85148700fa88095e3492d3f9f5beb43e555e5ff26d95f5a6adc36f8e6",
            },
        }
    });
    assert_let!(
        Ok(CanonicalJsonValue::Object(original_object)) =
            CanonicalJsonValue::try_from(original_event)
    );

    // The content is removed.
    let expected = json!({
        "content": {},
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 1_432_735,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "type": "local.custom.type",
        "hashes": {
            "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        },
        "signatures": {
            "example.org": {
                "ed25519:10": "cd372fb85148700fa88095e3492d3f9f5beb43e555e5ff26d95f5a6adc36f8e6",
            },
        }
    });
    let redacted = redact(original_object.clone(), &RedactionRules::V11, None).unwrap();
    assert_to_canonical_json_eq!(redacted, expected.clone());
    let redacted_canonical =
        RedactingSerializer::new().rules(&RedactionRules::V11).serialize(&original_object).unwrap();
    assert_eq!(redacted_canonical, to_json_string(&expected).unwrap());

    // Remove other keys.
    let expected = json!({
        "content": {},
        "event_id": "$143273582443PhrSn",
        "origin_server_ts": 1_432_735,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "type": "local.custom.type",
    });
    let redacted_canonical = RedactingSerializer::new()
        .rules(&RedactionRules::V11)
        .custom_redacted_root_fields(&["hashes", "signatures"])
        .serialize(&original_object)
        .unwrap();
    assert_eq!(redacted_canonical, to_json_string(&expected).unwrap());
}
