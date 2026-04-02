use assert_matches2::assert_let;
use ruma_common::{
    canonical_json::{CanonicalJsonValue, assert_to_canonical_json_eq, redact},
    room_version_rules::RedactionRules,
};
use serde_json::json;

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
    let redacted = redact(original_object.clone(), &RedactionRules::V1, None).unwrap();
    assert_to_canonical_json_eq!(
        redacted,
        json!({
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
        })
    );

    // In room v11, the `invite` key is also kept.
    let redacted = redact(original_object, &RedactionRules::V11, None).unwrap();
    assert_to_canonical_json_eq!(
        redacted,
        json!({
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
        })
    );
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
    let redacted = redact(original_object.clone(), &RedactionRules::V1, None).unwrap();
    assert_to_canonical_json_eq!(
        redacted,
        json!({
            "content": {
                "aliases": ["#somewhere:localhost"]
            },
            "event_id": "$152037280074GZeOm:localhost",
            "origin_server_ts": 1,
            "sender": "@example:localhost",
            "state_key": "room.com",
            "room_id": "!room:room.com",
            "type": "m.room.aliases",
        })
    );

    // In room v11, all the content keys are removed.
    let redacted = redact(original_object, &RedactionRules::V9, None).unwrap();
    assert_to_canonical_json_eq!(
        redacted,
        json!({
            "content": {},
            "event_id": "$152037280074GZeOm:localhost",
            "origin_server_ts": 1,
            "sender": "@example:localhost",
            "state_key": "room.com",
            "room_id": "!room:room.com",
            "type": "m.room.aliases",
        })
    );
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
    let redacted = redact(original_object.clone(), &RedactionRules::V1, None).unwrap();
    assert_to_canonical_json_eq!(
        redacted,
        json!({
            "content": {
                "creator": "@example:example.org",
            },
            "event_id": "$143273582443PhrSn",
            "origin_server_ts": 1_432_735,
            "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
            "sender": "@example:example.org",
            "state_key": "",
            "type": "m.room.create",
        })
    );

    // In room v11, all the content keys are kept.
    let redacted = redact(original_object, &RedactionRules::V11, None).unwrap();
    assert_to_canonical_json_eq!(
        redacted,
        json!({
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
        })
    );
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
    let redacted = redact(original_object.clone(), &RedactionRules::V1, None).unwrap();
    assert_to_canonical_json_eq!(
        redacted,
        json!({
            "content": {
                "membership": "invite",
            },
            "event_id": "$143273582443PhrSn",
            "origin_server_ts": 1_432_735,
            "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
            "sender": "@example:example.org",
            "state_key": "@alice:example.org",
            "type": "m.room.member",
        })
    );

    // In room v11, `membership` and `third_party_invite.signed` are kept.
    let redacted = redact(original_object, &RedactionRules::V11, None).unwrap();
    assert_to_canonical_json_eq!(
        redacted,
        json!({
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
        })
    );
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
    let redacted = redact(original_object.clone(), &RedactionRules::V1, None).unwrap();
    assert_to_canonical_json_eq!(
        redacted,
        json!({
            "content": {
                "membership": "join",
            },
            "event_id": "$143273582443PhrSn",
            "origin_server_ts": 1_432_735,
            "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
            "sender": "@example:example.org",
            "state_key": "@example:example.org",
            "type": "m.room.member",
        })
    );

    // In room v11, `membership` and `join_authorised_via_users_server` are kept.
    let redacted = redact(original_object, &RedactionRules::V11, None).unwrap();
    assert_to_canonical_json_eq!(
        redacted,
        json!({
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
        })
    );
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
    let redacted = redact(original_object.clone(), &RedactionRules::V1, None).unwrap();
    assert_to_canonical_json_eq!(
        redacted,
        json!({
            "content": {
                "join_rule": "restricted",
            },
            "event_id": "$143273582443PhrSn",
            "origin_server_ts": 1_432_735,
            "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
            "sender": "@example:example.org",
            "state_key": "",
            "type": "m.room.join_rules",
        })
    );

    // In v11, `join_rule` and `allow` are kept.
    let redacted = redact(original_object, &RedactionRules::V11, None).unwrap();
    assert_to_canonical_json_eq!(
        redacted,
        json!({
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
        })
    );
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
    let redacted = redact(original_object, &RedactionRules::V1, None).unwrap();
    assert_to_canonical_json_eq!(
        redacted,
        json!({
            "content": {
                "history_visibility": "invited",
            },
            "event_id": "$143273582443PhrSn",
            "origin_server_ts": 1_432_735,
            "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
            "sender": "@example:example.org",
            "state_key": "",
            "type": "m.room.history_visibility",
        })
    );
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
    let redacted = redact(original_object.clone(), &RedactionRules::V1, None).unwrap();
    assert_to_canonical_json_eq!(
        redacted,
        json!({
            "content": {},
            "event_id": "$143273582443PhrSn",
            "origin_server_ts": 1_432_735,
            "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
            "sender": "@example:example.org",
            "type": "m.room.redaction",
        })
    );

    // In v11, the `redacts` key is kept.
    let redacted = redact(original_object, &RedactionRules::V11, None).unwrap();
    assert_to_canonical_json_eq!(
        redacted,
        json!({
            "content": {
                "redacts": "$fukweghifu23",
            },
            "event_id": "$143273582443PhrSn",
            "origin_server_ts": 1_432_735,
            "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
            "sender": "@example:example.org",
            "type": "m.room.redaction",
        })
    );
}
