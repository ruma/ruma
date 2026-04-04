use std::collections::HashSet;

use js_int::int;
use ruma_common::{
    UserId, owned_event_id, room_version_rules::AuthorizationRules, serde::JsonObject,
};
use ruma_events::{TimelineEventType, room::power_levels::UserPowerLevel};
use serde_json::{Value as JsonValue, json};
use test_log::test;
use tracing::info;

use crate::{
    event_auth::check_room_power_levels,
    events::RoomPowerLevelsEvent,
    test_utils::{Pdu, UserFactory},
};

/// The default `m.room.power_levels` event content when creating a room with the given
/// authorization rules.
fn initial_room_power_levels_content(
    authorization_rules: &AuthorizationRules,
    creator: &UserId,
) -> JsonObject {
    let mut content = JsonObject::new();

    if !authorization_rules.explicitly_privilege_room_creators {
        let users = JsonObject::from_iter([(creator.to_string(), 100.into())]);
        content.insert("users".to_owned(), users.into());
    }

    content
}

/// The default `m.room.power_levels` PDU when creating a room with the given authorization rules.
fn initial_room_power_levels(authorization_rules: &AuthorizationRules) -> Pdu {
    let creator = UserFactory::Alice.user_id();
    let content = initial_room_power_levels_content(authorization_rules, &creator);

    Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-initial"),
        creator,
        TimelineEventType::RoomPowerLevels,
        String::new(),
        content,
    )
}

#[test]
fn not_int_or_string_int_in_content() {
    let current_room_power_levels_event = initial_room_power_levels(&AuthorizationRules::V6);
    let creator = current_room_power_levels_event.sender.clone();

    let int_fields =
        &["users_default", "events_default", "state_default", "ban", "redact", "kick", "invite"];

    // Tuples of (is_string, is_int) booleans.
    let combinations = &[(true, false), (true, true), (false, true)];

    for field in int_fields {
        for (is_string, is_int) in combinations {
            info!(?field, ?is_string, ?is_int, "checking field");

            let value: JsonValue = match (is_string, is_int) {
                (true, false) => "foo".into(),
                (true, true) => "50".into(),
                (false, true) => 50.into(),
                _ => unreachable!(),
            };

            let mut content = initial_room_power_levels_content(&AuthorizationRules::V6, &creator);
            content.insert((*field).to_owned(), value.clone());

            let pdu = Pdu::with_minimal_state_fields(
                owned_event_id!("$room-power-levels-field"),
                creator.clone(),
                TimelineEventType::RoomPowerLevels,
                String::new(),
                content,
            );

            // String that is not a number is not accepted.
            let v6_result = check_room_power_levels(
                RoomPowerLevelsEvent::new(&pdu),
                Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
                &AuthorizationRules::V6,
                int!(100).into(),
                &HashSet::new(),
            );

            if *is_int {
                v6_result.unwrap();
            } else {
                assert_eq!(
                    v6_result.unwrap_err(),
                    format!(
                        "unexpected format of `{field}` field in `content` of `m.room.power_levels` event: invalid digit found in string"
                    )
                );
            }

            // String is not accepted.
            let v10_result = check_room_power_levels(
                RoomPowerLevelsEvent::new(&pdu),
                Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
                &AuthorizationRules::V10,
                int!(100).into(),
                &HashSet::new(),
            );

            if *is_string {
                assert_eq!(
                    v10_result.unwrap_err(),
                    format!(
                        "unexpected format of `{field}` field in `content` of `m.room.power_levels` event: invalid type: string \"{}\", expected i64",
                        value.as_str().unwrap()
                    )
                );
            } else {
                v10_result.unwrap();
            }
        }
    }
}

#[test]
fn not_int_or_string_int_in_events() {
    let current_room_power_levels_event = initial_room_power_levels(&AuthorizationRules::V6);
    let creator = current_room_power_levels_event.sender.clone();

    // Tuples of (is_string, is_int) booleans.
    let combinations = &[(true, false), (true, true), (false, true)];

    for (is_string, is_int) in combinations {
        info!(?is_string, ?is_int, "checking field");

        let value: JsonValue = match (is_string, is_int) {
            (true, false) => "foo".into(),
            (true, true) => "50".into(),
            (false, true) => 50.into(),
            _ => unreachable!(),
        };
        let mut events = JsonObject::new();
        events.insert("bar".to_owned(), value.clone());

        let mut content = initial_room_power_levels_content(&AuthorizationRules::V6, &creator);
        content.insert("events".to_owned(), events.into());

        let pdu = Pdu::with_minimal_state_fields(
            owned_event_id!("$room-power-levels-events"),
            creator.clone(),
            TimelineEventType::RoomPowerLevels,
            String::new(),
            content,
        );

        // String that is not a number is not accepted.
        let v6_result = check_room_power_levels(
            RoomPowerLevelsEvent::new(&pdu),
            Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
            &AuthorizationRules::V6,
            int!(100).into(),
            &HashSet::new(),
        );

        if *is_int {
            v6_result.unwrap();
        } else {
            assert_eq!(
                v6_result.unwrap_err(),
                "unexpected format of `events` field in `content` of `m.room.power_levels` event: invalid digit found in string"
            );
        }

        // String is not accepted.
        let v10_result = check_room_power_levels(
            RoomPowerLevelsEvent::new(&pdu),
            Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
            &AuthorizationRules::V10,
            int!(100).into(),
            &HashSet::new(),
        );

        if *is_string {
            assert_eq!(
                v10_result.unwrap_err(),
                format!(
                    "unexpected format of `events` field in `content` of `m.room.power_levels` event: invalid type: string \"{}\", expected i64",
                    value.as_str().unwrap()
                )
            );
        } else {
            v10_result.unwrap();
        }
    }
}

#[test]
fn not_int_or_string_int_in_notifications() {
    let current_room_power_levels_event = initial_room_power_levels(&AuthorizationRules::V6);
    let creator = current_room_power_levels_event.sender.clone();

    // Tuples of (is_string, is_int) booleans.
    let combinations = &[(true, false), (true, true), (false, true)];

    for (is_string, is_int) in combinations {
        info!(?is_string, ?is_int, "checking field");

        let value: JsonValue = match (is_string, is_int) {
            (true, false) => "foo".into(),
            (true, true) => "50".into(),
            (false, true) => 50.into(),
            _ => unreachable!(),
        };
        let mut notifications = JsonObject::new();
        notifications.insert("room".to_owned(), value.clone());

        let mut content = initial_room_power_levels_content(&AuthorizationRules::V6, &creator);
        content.insert("notifications".to_owned(), notifications.into());

        let pdu = Pdu::with_minimal_state_fields(
            owned_event_id!("$room-power-levels-notifications"),
            creator.clone(),
            TimelineEventType::RoomPowerLevels,
            String::new(),
            content,
        );

        // String that is not a number is not accepted.
        let v6_result = check_room_power_levels(
            RoomPowerLevelsEvent::new(&pdu),
            Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
            &AuthorizationRules::V6,
            int!(100).into(),
            &HashSet::new(),
        );

        if *is_int {
            v6_result.unwrap();
        } else {
            assert_eq!(
                v6_result.unwrap_err(),
                "unexpected format of `notifications` field in `content` of `m.room.power_levels` event: invalid digit found in string"
            );
        }

        // String is not accepted.
        let v10_result = check_room_power_levels(
            RoomPowerLevelsEvent::new(&pdu),
            Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
            &AuthorizationRules::V10,
            int!(100).into(),
            &HashSet::new(),
        );

        if *is_string {
            assert_eq!(
                v10_result.unwrap_err(),
                format!(
                    "unexpected format of `notifications` field in `content` of `m.room.power_levels` event: invalid type: string \"{}\", expected i64",
                    value.as_str().unwrap()
                )
            );
        } else {
            v10_result.unwrap();
        }
    }
}

#[test]
fn not_user_id_in_users() {
    let current_room_power_levels_event = initial_room_power_levels(&AuthorizationRules::V6);
    let creator = current_room_power_levels_event.sender.clone();

    let pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-spambot"),
        creator.clone(),
        TimelineEventType::RoomPowerLevels,
        String::new(),
        json!({
            "users": {
                creator: 100,
                "spambot": -1,
            },
        }),
    );

    // Key that is not a user ID is not accepted.
    assert_eq!(
        check_room_power_levels(
            RoomPowerLevelsEvent::new(pdu),
            Some(RoomPowerLevelsEvent::new(current_room_power_levels_event)),
            &AuthorizationRules::V6,
            int!(100).into(),
            &HashSet::new(),
        )
        .unwrap_err(),
        "unexpected format of `users` field in `content` of `m.room.power_levels` event: leading sigil is incorrect or missing"
    );
}

#[test]
fn not_int_or_string_int_in_users() {
    let current_room_power_levels_event = initial_room_power_levels(&AuthorizationRules::V6);
    let creator = current_room_power_levels_event.sender.clone();

    // Tuples of (is_string, is_int) booleans.
    let combinations = &[(true, false), (true, true), (false, true)];

    for (is_string, is_int) in combinations {
        info!(?is_string, ?is_int, "checking field");

        let value: JsonValue = match (is_string, is_int) {
            (true, false) => "foo".into(),
            (true, true) => "50".into(),
            (false, true) => 50.into(),
            _ => unreachable!(),
        };

        let mut content = initial_room_power_levels_content(&AuthorizationRules::V6, &creator);
        let users = content.get_mut("users").unwrap().as_object_mut().unwrap();
        users.insert("@bar:baz".to_owned(), value.clone());

        let incoming_event = Pdu::with_minimal_state_fields(
            owned_event_id!("$room-power-levels-bar"),
            creator.clone(),
            TimelineEventType::RoomPowerLevels,
            String::new(),
            content,
        );

        // String that is not a number is not accepted.
        let v6_result = check_room_power_levels(
            RoomPowerLevelsEvent::new(&incoming_event),
            Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
            &AuthorizationRules::V6,
            int!(100).into(),
            &HashSet::new(),
        );

        if *is_int {
            v6_result.unwrap();
        } else {
            assert_eq!(
                v6_result.unwrap_err(),
                "unexpected format of `users` field in `content` of `m.room.power_levels` event: invalid digit found in string"
            );
        }

        // String is not accepted.
        let v10_result = check_room_power_levels(
            RoomPowerLevelsEvent::new(&incoming_event),
            Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
            &AuthorizationRules::V10,
            int!(100).into(),
            &HashSet::new(),
        );

        if *is_string {
            assert_eq!(
                v10_result.unwrap_err(),
                format!(
                    "unexpected format of `users` field in `content` of `m.room.power_levels` event: invalid type: string \"{}\", expected i64",
                    value.as_str().unwrap()
                )
            );
        } else {
            v10_result.unwrap();
        }
    }
}

#[test]
fn first_power_levels_event() {
    // First power levels event is accepted.
    check_room_power_levels(
        RoomPowerLevelsEvent::new(&initial_room_power_levels(&AuthorizationRules::V6)),
        None::<RoomPowerLevelsEvent<&Pdu>>,
        &AuthorizationRules::V6,
        int!(100).into(),
        &HashSet::new(),
    )
    .unwrap();
}

#[test]
fn change_content_level_with_current_higher_power_level() {
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();

    let original_content = JsonObject::from_iter([(
        "users".to_owned(),
        json!({
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
        }),
    )]);

    let int_fields =
        &["users_default", "events_default", "state_default", "ban", "redact", "kick", "invite"];

    for field in int_fields {
        info!(?field, "checking field");

        let current_value = 60;
        let incoming_value = 40;

        let mut current_content = original_content.clone();
        current_content.insert((*field).to_owned(), current_value.into());

        let current_room_power_levels_event = Pdu::with_minimal_state_fields(
            owned_event_id!("$room-power-levels-current"),
            alice_id.clone(),
            TimelineEventType::RoomPowerLevels,
            String::new(),
            current_content,
        );

        let mut incoming_content = original_content.clone();
        incoming_content.insert((*field).to_owned(), incoming_value.into());

        let pdu = Pdu::with_minimal_state_fields(
            owned_event_id!("$room-power-levels-field"),
            bob_id.clone(),
            TimelineEventType::RoomPowerLevels,
            String::new(),
            incoming_content,
        );

        // Cannot change from a power level that is higher than the user.
        assert_eq!(
            check_room_power_levels(
                RoomPowerLevelsEvent::new(&pdu),
                Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
                &AuthorizationRules::V6,
                int!(40).into(),
                &HashSet::new(),
            )
            .unwrap_err(),
            format!("sender does not have enough power to change the power level of `{field}`")
        );
    }
}

#[test]
fn change_content_level_with_new_higher_power_level() {
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();

    let original_content = JsonObject::from_iter([(
        "users".to_owned(),
        json!({
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
        }),
    )]);

    let int_fields =
        &["users_default", "events_default", "state_default", "ban", "redact", "kick", "invite"];

    for field in int_fields {
        info!(?field, "checking field");

        let current_value = 40;
        let incoming_value = 60;

        let mut current_content = original_content.clone();
        current_content.insert((*field).to_owned(), current_value.into());

        let current_room_power_levels_event = Pdu::with_minimal_state_fields(
            owned_event_id!("$room-power-levels-current"),
            alice_id.clone(),
            TimelineEventType::RoomPowerLevels,
            String::new(),
            current_content,
        );

        let mut incoming_content = original_content.clone();
        incoming_content.insert((*field).to_owned(), incoming_value.into());

        let pdu = Pdu::with_minimal_state_fields(
            owned_event_id!("$room-power-levels-field"),
            bob_id.clone(),
            TimelineEventType::RoomPowerLevels,
            String::new(),
            incoming_content,
        );

        // Cannot change to a power level that is higher than the user.
        assert_eq!(
            check_room_power_levels(
                RoomPowerLevelsEvent::new(&pdu),
                Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
                &AuthorizationRules::V6,
                int!(40).into(),
                &HashSet::new(),
            )
            .unwrap_err(),
            format!("sender does not have enough power to change the power level of `{field}`")
        );
    }
}

#[test]
fn change_content_level_with_same_power_level() {
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();

    let original_content = JsonObject::from_iter([(
        "users".to_owned(),
        json!({
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
        }),
    )]);

    let int_fields =
        &["users_default", "events_default", "state_default", "ban", "redact", "kick", "invite"];

    for field in int_fields {
        info!(?field, "checking field");

        let current_value = 30;
        let incoming_value = 40;

        let mut current_content = original_content.clone();
        current_content.insert((*field).to_owned(), current_value.into());

        let current_room_power_levels_event = Pdu::with_minimal_state_fields(
            owned_event_id!("$room-power-levels-current"),
            alice_id.clone(),
            TimelineEventType::RoomPowerLevels,
            String::new(),
            current_content,
        );

        let mut incoming_content = original_content.clone();
        incoming_content.insert((*field).to_owned(), incoming_value.into());

        let pdu = Pdu::with_minimal_state_fields(
            owned_event_id!("$room-power-levels-field"),
            bob_id.clone(),
            TimelineEventType::RoomPowerLevels,
            String::new(),
            incoming_content,
        );

        // Can change a power level that is the same or lower than the user.
        check_room_power_levels(
            RoomPowerLevelsEvent::new(&pdu),
            Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
            &AuthorizationRules::V6,
            int!(40).into(),
            &HashSet::new(),
        )
        .unwrap();
    }
}

#[test]
fn change_events_level_with_current_higher_power_level() {
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();

    let current_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
        },
        "events": {
            "foo": 60,
        },
    });
    let current_room_power_levels_event = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-current"),
        alice_id.clone(),
        TimelineEventType::RoomPowerLevels,
        String::new(),
        current_content,
    );

    let incoming_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
        },
        "events": {
            "foo": 40,
        },
    });
    let pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-incoming"),
        bob_id,
        TimelineEventType::RoomPowerLevels,
        String::new(),
        incoming_content,
    );

    // Cannot change from a power level that is higher than the user.
    assert_eq!(
        check_room_power_levels(
            RoomPowerLevelsEvent::new(&pdu),
            Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
            &AuthorizationRules::V6,
            int!(40).into(),
            &HashSet::new(),
        )
        .unwrap_err(),
        "sender does not have enough power to change the `foo` event type power level"
    );
}

#[test]
fn change_events_level_with_new_higher_power_level() {
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();

    let current_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
        },
        "events": {
            "foo": 30,
        },
    });
    let current_room_power_levels_event = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-current"),
        alice_id.clone(),
        TimelineEventType::RoomPowerLevels,
        String::new(),
        current_content,
    );

    let incoming_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
        },
        "events": {
            "foo": 60,
        },
    });
    let pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-incoming"),
        bob_id,
        TimelineEventType::RoomPowerLevels,
        String::new(),
        incoming_content,
    );

    // Cannot change to a power level that is higher than the user.
    assert_eq!(
        check_room_power_levels(
            RoomPowerLevelsEvent::new(&pdu),
            Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
            &AuthorizationRules::V6,
            int!(40).into(),
            &HashSet::new(),
        )
        .unwrap_err(),
        "sender does not have enough power to change the `foo` event type power level"
    );
}

#[test]
fn change_events_level_with_same_power_level() {
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();

    let current_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
        },
        "events": {
            "foo": 40,
        },
    });
    let current_room_power_levels_event = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-current"),
        alice_id.clone(),
        TimelineEventType::RoomPowerLevels,
        String::new(),
        current_content,
    );

    let incoming_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
        },
        "events": {
            "foo": 10,
        },
    });
    let pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-incoming"),
        bob_id,
        TimelineEventType::RoomPowerLevels,
        String::new(),
        incoming_content,
    );

    // Can change a power level that is the same or lower than the user.
    check_room_power_levels(
        RoomPowerLevelsEvent::new(&pdu),
        Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
        &AuthorizationRules::V6,
        int!(40).into(),
        &HashSet::new(),
    )
    .unwrap();
}

#[test]
fn change_notifications_level_with_current_higher_power_level() {
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();

    let current_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
        },
        "notifications": {
            "room": 60,
        },
    });
    let current_room_power_levels_event = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-current"),
        alice_id.clone(),
        TimelineEventType::RoomPowerLevels,
        String::new(),
        current_content,
    );

    let incoming_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
        },
        "notifications": {
            "room": 40,
        },
    });
    let pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-incoming"),
        bob_id,
        TimelineEventType::RoomPowerLevels,
        String::new(),
        incoming_content,
    );

    // Notifications are not checked before v6.
    check_room_power_levels(
        RoomPowerLevelsEvent::new(&pdu),
        Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
        &AuthorizationRules::V3,
        int!(40).into(),
        &HashSet::new(),
    )
    .unwrap();

    // Cannot change from a power level that is higher than the user.
    assert_eq!(
        check_room_power_levels(
            RoomPowerLevelsEvent::new(&pdu),
            Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
            &AuthorizationRules::V6,
            int!(40).into(),
            &HashSet::new(),
        )
        .unwrap_err(),
        "sender does not have enough power to change the `room` notification power level"
    );
}

#[test]
fn change_notifications_level_with_new_higher_power_level() {
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();

    let current_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
        },
        "notifications": {
            "room": 30,
        },
    });
    let current_room_power_levels_event = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-current"),
        alice_id.clone(),
        TimelineEventType::RoomPowerLevels,
        String::new(),
        current_content,
    );

    let incoming_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
        },
        "notifications": {
            "room": 60,
        },
    });
    let pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-incoming"),
        bob_id,
        TimelineEventType::RoomPowerLevels,
        String::new(),
        incoming_content,
    );

    // Notifications are not checked before v6.
    check_room_power_levels(
        RoomPowerLevelsEvent::new(&pdu),
        Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
        &AuthorizationRules::V3,
        int!(40).into(),
        &HashSet::new(),
    )
    .unwrap();

    // Cannot change to a power level that is higher than the user.
    assert_eq!(
        check_room_power_levels(
            RoomPowerLevelsEvent::new(&pdu),
            Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
            &AuthorizationRules::V6,
            int!(40).into(),
            &HashSet::new(),
        )
        .unwrap_err(),
        "sender does not have enough power to change the `room` notification power level"
    );
}

#[test]
fn change_notifications_level_with_same_power_level() {
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();

    let current_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
        },
        "notifications": {
            "room": 30,
        },
    });
    let current_room_power_levels_event = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-current"),
        alice_id.clone(),
        TimelineEventType::RoomPowerLevels,
        String::new(),
        current_content,
    );

    let incoming_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
        },
        "notifications": {
            "room": 31,
        },
    });
    let pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-incoming"),
        bob_id,
        TimelineEventType::RoomPowerLevels,
        String::new(),
        incoming_content,
    );

    // Notifications are not checked before v6.
    check_room_power_levels(
        RoomPowerLevelsEvent::new(&pdu),
        Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
        &AuthorizationRules::V3,
        int!(40).into(),
        &HashSet::new(),
    )
    .unwrap();

    // Can change a power level that is the same or lower than the user.
    check_room_power_levels(
        RoomPowerLevelsEvent::new(&pdu),
        Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
        &AuthorizationRules::V6,
        int!(40).into(),
        &HashSet::new(),
    )
    .unwrap();
}

#[test]
fn change_other_user_level_with_current_higher_power_level() {
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();
    let zara_id = UserFactory::Zara.user_id();

    let current_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
            zara_id.to_string(): 70,
        },
    });
    let current_room_power_levels_event = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-current"),
        alice_id.clone(),
        TimelineEventType::RoomPowerLevels,
        String::new(),
        current_content,
    );

    let incoming_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
        },
    });
    let pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-incoming"),
        bob_id,
        TimelineEventType::RoomPowerLevels,
        String::new(),
        incoming_content,
    );

    // Cannot change from a power level that is higher than the user.
    assert_eq!(
        check_room_power_levels(
            RoomPowerLevelsEvent::new(&pdu),
            Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
            &AuthorizationRules::V6,
            int!(40).into(),
            &HashSet::new(),
        )
        .unwrap_err(),
        "sender does not have enough power to change `@zara:other.local`'s  power level"
    );
}

#[test]
fn change_other_user_level_with_new_higher_power_level() {
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();
    let zara_id = UserFactory::Zara.user_id();

    let current_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
            zara_id.to_string(): 10,
        },
    });
    let current_room_power_levels_event = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-current"),
        alice_id.clone(),
        TimelineEventType::RoomPowerLevels,
        String::new(),
        current_content,
    );

    let incoming_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
            zara_id.to_string(): 45,
        },
    });
    let pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-incoming"),
        bob_id,
        TimelineEventType::RoomPowerLevels,
        String::new(),
        incoming_content,
    );

    // Cannot change to a power level that is higher than the user.
    assert_eq!(
        check_room_power_levels(
            RoomPowerLevelsEvent::new(&pdu),
            Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
            &AuthorizationRules::V6,
            int!(40).into(),
            &HashSet::new(),
        )
        .unwrap_err(),
        "sender does not have enough power to change `@zara:other.local`'s  power level"
    );
}

#[test]
fn change_other_user_level_with_same_power_level() {
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();
    let zara_id = UserFactory::Zara.user_id();

    let current_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
            zara_id.to_string(): 20,
        },
    });
    let current_room_power_levels_event = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-current"),
        alice_id.clone(),
        TimelineEventType::RoomPowerLevels,
        String::new(),
        current_content,
    );

    let incoming_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
            zara_id.to_string(): 40,
        },
    });
    let pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-incoming"),
        bob_id,
        TimelineEventType::RoomPowerLevels,
        String::new(),
        incoming_content,
    );

    // Can change a power level that is the same or lower than the user.
    check_room_power_levels(
        RoomPowerLevelsEvent::new(&pdu),
        Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
        &AuthorizationRules::V6,
        int!(40).into(),
        &HashSet::new(),
    )
    .unwrap();
}

#[test]
fn change_own_user_level_to_new_higher_power_level() {
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();

    let current_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
        },
    });
    let current_room_power_levels_event = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-current"),
        alice_id.clone(),
        TimelineEventType::RoomPowerLevels,
        String::new(),
        current_content,
    );

    let incoming_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 100,
        },
    });
    let pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-incoming"),
        bob_id,
        TimelineEventType::RoomPowerLevels,
        String::new(),
        incoming_content,
    );

    // Cannot change to a power level that is higher than the user.
    assert_eq!(
        check_room_power_levels(
            RoomPowerLevelsEvent::new(&pdu),
            Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
            &AuthorizationRules::V6,
            int!(40).into(),
            &HashSet::new(),
        )
        .unwrap_err(),
        "sender does not have enough power to change `@bob:matrix.local`'s  power level"
    );
}

#[test]
fn change_own_user_level_to_lower_power_level() {
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();

    let current_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 40,
        },
    });
    let current_room_power_levels_event = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-current"),
        alice_id.clone(),
        TimelineEventType::RoomPowerLevels,
        String::new(),
        current_content,
    );

    let incoming_content = json!({
        "users": {
            alice_id.to_string(): 100,
            bob_id.to_string(): 20,
        },
    });
    let pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-incoming"),
        bob_id,
        TimelineEventType::RoomPowerLevels,
        String::new(),
        incoming_content,
    );

    // Can change own power level to a lower level than the user.
    check_room_power_levels(
        RoomPowerLevelsEvent::new(&pdu),
        Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
        &AuthorizationRules::V6,
        int!(40).into(),
        &HashSet::new(),
    )
    .unwrap();
}

#[test]
fn creator_has_infinite_power() {
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();

    let current_content = json!({
        "users": {
            bob_id.to_string(): js_int::Int::MAX,
        },
    });
    let current_room_power_levels_event = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-current"),
        alice_id.clone(),
        TimelineEventType::RoomPowerLevels,
        String::new(),
        current_content,
    );

    let incoming_content = json!({
        "users": {
            bob_id.to_string(): 0,
        },
    });
    let pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-incoming"),
        alice_id,
        TimelineEventType::RoomPowerLevels,
        String::new(),
        incoming_content,
    );

    // Room creator has infinite power level, and hence can change the power level of any other
    // user.
    check_room_power_levels(
        RoomPowerLevelsEvent::new(&pdu),
        Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
        &AuthorizationRules::V12,
        UserPowerLevel::Infinite,
        &HashSet::new(),
    )
    .unwrap();
}

#[test]
fn dont_allow_creator_in_users_field() {
    let alice_id = UserFactory::Alice.user_id();
    let bob_id = UserFactory::Bob.user_id();

    let current_content = json!({
        "users": {
            bob_id.to_string(): 40,
        },
    });
    let current_room_power_levels_event = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-current"),
        alice_id.clone(),
        TimelineEventType::RoomPowerLevels,
        String::new(),
        current_content,
    );

    let incoming_content = json!({
        "users": {
            alice_id.to_string(): 10,
            bob_id.to_string(): 40,
        },
    });
    let pdu = Pdu::with_minimal_state_fields(
        owned_event_id!("$room-power-levels-incoming"),
        alice_id.clone(),
        TimelineEventType::RoomPowerLevels,
        String::new(),
        incoming_content,
    );

    // Room creator cannot be in the `users` field of the power levels event
    assert_eq!(
        check_room_power_levels(
            RoomPowerLevelsEvent::new(&pdu),
            Some(RoomPowerLevelsEvent::new(&current_room_power_levels_event)),
            &AuthorizationRules::V12,
            UserPowerLevel::Infinite,
            &HashSet::from_iter([alice_id]),
        )
        .unwrap_err(),
        "creator user IDs are not allowed in the `users` field"
    );
}
