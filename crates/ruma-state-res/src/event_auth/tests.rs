use js_int::int;
use ruma_common::{
    owned_event_id, owned_room_alias_id, room_version_rules::AuthorizationRules, user_id,
};
use ruma_events::{
    room::{
        aliases::RoomAliasesEventContent, message::RoomMessageEventContent,
        redaction::RoomRedactionEventContent,
    },
    TimelineEventType,
};
use serde_json::{json, value::to_raw_value as to_raw_json_value};

mod room_power_levels;

use self::room_power_levels::default_room_power_levels;
use super::check_room_create;
use crate::{
    auth_check,
    event_auth::check_room_redaction,
    events::{RoomCreateEvent, RoomPowerLevelsEvent},
    test_utils::{
        alice, charlie, ella, event_id, init_subscriber, member_content_join,
        room_redaction_pdu_event, room_third_party_invite, to_init_pdu_event, to_pdu_event,
        TestStateMap, INITIAL_EVENTS,
    },
};

#[test]
fn valid_room_create() {
    // Minimal fields valid for room v1.
    let content = json!({
        "creator": alice(),
    });
    let event = to_init_pdu_event(
        "CREATE",
        alice(),
        TimelineEventType::RoomCreate,
        Some(""),
        to_raw_json_value(&content).unwrap(),
    );
    check_room_create(RoomCreateEvent::new(event), &AuthorizationRules::V1).unwrap();

    // Same, with room version.
    let content = json!({
        "creator": alice(),
        "room_version": "2",
    });
    let event = to_init_pdu_event(
        "CREATE",
        alice(),
        TimelineEventType::RoomCreate,
        Some(""),
        to_raw_json_value(&content).unwrap(),
    );
    check_room_create(RoomCreateEvent::new(event), &AuthorizationRules::V1).unwrap();

    // With a room version that does not need the creator.
    let content = json!({
        "room_version": "11",
    });
    let event = to_init_pdu_event(
        "CREATE",
        alice(),
        TimelineEventType::RoomCreate,
        Some(""),
        to_raw_json_value(&content).unwrap(),
    );
    check_room_create(RoomCreateEvent::new(event), &AuthorizationRules::V11).unwrap();

    // Check various contents that might not match the definition of `m.room.create` in the
    // spec, to ensure that we only care about a few fields.
    let contents_to_check = vec![
        // With an invalid predecessor, but we don't care about it. Inspired by a real-life
        // example.
        json!({
            "room_version": "11",
            "predecessor": "!XPoLiaavxVgyMSiRwK:localhost",
        }),
        // With an invalid type, but we don't care about it.
        json!({
            "room_version": "11",
            "type": true,
        }),
    ];

    for content in contents_to_check {
        let event = to_init_pdu_event(
            "CREATE",
            alice(),
            TimelineEventType::RoomCreate,
            Some(""),
            to_raw_json_value(&content).unwrap(),
        );
        check_room_create(RoomCreateEvent::new(event), &AuthorizationRules::V11).unwrap();
    }
}

#[test]
fn invalid_room_create() {
    // With a prev event.
    let content = json!({
        "creator": alice(),
    });
    let event = to_pdu_event(
        "CREATE",
        alice(),
        TimelineEventType::RoomCreate,
        Some(""),
        to_raw_json_value(&content).unwrap(),
        &["OTHER_CREATE"],
        &["OTHER_CREATE"],
    );
    check_room_create(RoomCreateEvent::new(event), &AuthorizationRules::V1).unwrap_err();

    // Sender with a different domain.
    let creator = user_id!("@bot:bar");
    let content = json!({
        "creator": creator,
    });
    let event = to_init_pdu_event(
        "CREATE",
        creator,
        TimelineEventType::RoomCreate,
        Some(""),
        to_raw_json_value(&content).unwrap(),
    );
    check_room_create(RoomCreateEvent::new(event), &AuthorizationRules::V1).unwrap_err();

    // No creator in v1.
    let content = json!({});
    let event = to_init_pdu_event(
        "CREATE",
        alice(),
        TimelineEventType::RoomCreate,
        Some(""),
        to_raw_json_value(&content).unwrap(),
    );
    check_room_create(RoomCreateEvent::new(event), &AuthorizationRules::V1).unwrap_err();
}

#[test]
fn redact_higher_power_level() {
    let _guard = init_subscriber();

    let incoming_event = room_redaction_pdu_event(
        "HELLO",
        charlie(),
        owned_event_id!("$redacted_event:other.server"),
        to_raw_json_value(&RoomRedactionEventContent::new_v1()).unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let room_power_levels_event = Some(default_room_power_levels());

    // Cannot redact if redact level is higher than user's.
    check_room_redaction(incoming_event, room_power_levels_event, &AuthorizationRules::V1, int!(0))
        .unwrap_err();
}

#[test]
fn redact_same_power_level() {
    let _guard = init_subscriber();

    let incoming_event = room_redaction_pdu_event(
        "HELLO",
        charlie(),
        owned_event_id!("$redacted_event:other.server"),
        to_raw_json_value(&RoomRedactionEventContent::new_v1()).unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let room_power_levels_event = Some(RoomPowerLevelsEvent::new(to_pdu_event(
        "IPOWER",
        alice(),
        TimelineEventType::RoomPowerLevels,
        Some(""),
        to_raw_json_value(&json!({ "users": { alice(): 100, charlie(): 50 } })).unwrap(),
        &["CREATE", "IMA"],
        &["IMA"],
    )));

    // Can redact if redact level is same as user's.
    check_room_redaction(
        incoming_event,
        room_power_levels_event,
        &AuthorizationRules::V1,
        int!(50),
    )
    .unwrap();
}

#[test]
fn redact_same_server() {
    let _guard = init_subscriber();

    let incoming_event = room_redaction_pdu_event(
        "HELLO",
        charlie(),
        event_id("redacted_event"),
        to_raw_json_value(&RoomRedactionEventContent::new_v1()).unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let room_power_levels_event = Some(default_room_power_levels());

    // Can redact if redact level is same as user's.
    check_room_redaction(incoming_event, room_power_levels_event, &AuthorizationRules::V1, int!(0))
        .unwrap();
}

#[test]
fn missing_room_create_in_state() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMessage,
        None,
        to_raw_json_value(&RoomMessageEventContent::text_plain("Hi!")).unwrap(),
        &["IMA", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    init_events.remove(&event_id("CREATE"));

    let auth_events = TestStateMap::new(&init_events);
    let fetch_state = auth_events.fetch_state_fn();

    // Cannot accept event if no `m.room.create` in state.
    auth_check(&AuthorizationRules::V6, incoming_event, fetch_state).unwrap_err();
}

#[test]
fn missing_room_create_auth_events() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMessage,
        None,
        to_raw_json_value(&RoomMessageEventContent::text_plain("Hi!")).unwrap(),
        &["IMA", "IPOWER"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = TestStateMap::new(&init_events);
    let fetch_state = auth_events.fetch_state_fn();

    // Cannot accept event if no `m.room.create` in auth events.
    auth_check(&AuthorizationRules::V6, incoming_event, fetch_state).unwrap_err();
}

#[test]
fn no_federate_different_server() {
    let _guard = init_subscriber();

    let sender = user_id!("@aya:other.server");
    let incoming_event = to_pdu_event(
        "AYA_JOIN",
        sender,
        TimelineEventType::RoomMember,
        Some(sender.as_str()),
        member_content_join(),
        &["CREATE", "IJR", "IPOWER"],
        &["IMB"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("CREATE")).unwrap() = to_init_pdu_event(
        "CREATE",
        alice(),
        TimelineEventType::RoomCreate,
        Some(""),
        to_raw_json_value(&json!({
            "creator": alice(),
            "m.federate": false,
        }))
        .unwrap(),
    );

    let auth_events = TestStateMap::new(&init_events);
    let fetch_state = auth_events.fetch_state_fn();

    // Cannot accept event if not federating and different server.
    auth_check(&AuthorizationRules::V6, incoming_event, fetch_state).unwrap_err();
}

#[test]
fn no_federate_same_server() {
    let _guard = init_subscriber();

    let sender = user_id!("@aya:foo");
    let incoming_event = to_pdu_event(
        "AYA_JOIN",
        sender,
        TimelineEventType::RoomMember,
        Some(sender.as_str()),
        member_content_join(),
        &["CREATE", "IJR", "IPOWER"],
        &["IMB"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("CREATE")).unwrap() = to_init_pdu_event(
        "CREATE",
        alice(),
        TimelineEventType::RoomCreate,
        Some(""),
        to_raw_json_value(&json!({
            "creator": alice(),
            "m.federate": false,
        }))
        .unwrap(),
    );

    let auth_events = TestStateMap::new(&init_events);
    let fetch_state = auth_events.fetch_state_fn();

    // Accept event if not federating and same server.
    auth_check(&AuthorizationRules::V6, incoming_event, fetch_state).unwrap();
}

#[test]
fn room_aliases_no_state_key() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "ALIASES",
        alice(),
        TimelineEventType::RoomAliases,
        None,
        to_raw_json_value(&RoomAliasesEventContent::new(vec![
            owned_room_alias_id!("#room:foo"),
            owned_room_alias_id!("#room_alt:foo"),
        ]))
        .unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IMB"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = TestStateMap::new(&init_events);
    let fetch_state = auth_events.fetch_state_fn();

    // Cannot accept `m.room.aliases` without state key.
    auth_check(&AuthorizationRules::V3, &incoming_event, fetch_state).unwrap_err();

    // `m.room.aliases` is not checked since v6.
    auth_check(&AuthorizationRules::V8, &incoming_event, fetch_state).unwrap();
}

#[test]
fn room_aliases_other_server() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "ALIASES",
        alice(),
        TimelineEventType::RoomAliases,
        Some("bar"),
        to_raw_json_value(&RoomAliasesEventContent::new(vec![
            owned_room_alias_id!("#room:bar"),
            owned_room_alias_id!("#room_alt:bar"),
        ]))
        .unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IMB"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = TestStateMap::new(&init_events);
    let fetch_state = auth_events.fetch_state_fn();

    // Cannot accept `m.room.aliases` with different server name than sender.
    auth_check(&AuthorizationRules::V3, &incoming_event, fetch_state).unwrap_err();

    // `m.room.aliases` is not checked since v6.
    auth_check(&AuthorizationRules::V8, &incoming_event, fetch_state).unwrap();
}

#[test]
fn room_aliases_same_server() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "ALIASES",
        alice(),
        TimelineEventType::RoomAliases,
        Some("foo"),
        to_raw_json_value(&RoomAliasesEventContent::new(vec![
            owned_room_alias_id!("#room:foo"),
            owned_room_alias_id!("#room_alt:foo"),
        ]))
        .unwrap(),
        &["CREATE", "IJR", "IPOWER"],
        &["IMB"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = TestStateMap::new(&init_events);
    let fetch_state = auth_events.fetch_state_fn();

    // Accept `m.room.aliases` with same server name as sender.
    auth_check(&AuthorizationRules::V3, &incoming_event, fetch_state).unwrap();

    // `m.room.aliases` is not checked since v6.
    auth_check(&AuthorizationRules::V8, &incoming_event, fetch_state).unwrap();
}

#[test]
fn sender_not_in_room() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        ella(),
        TimelineEventType::RoomMessage,
        None,
        to_raw_json_value(&RoomMessageEventContent::text_plain("Hi!")).unwrap(),
        &["IMA", "IPOWER", "CREATE"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = TestStateMap::new(&init_events);
    let fetch_state = auth_events.fetch_state_fn();

    // Cannot accept event if user not in room.
    auth_check(&AuthorizationRules::V6, incoming_event, fetch_state).unwrap_err();
}

#[test]
fn room_third_party_invite_not_enough_power() {
    let _guard = init_subscriber();

    let incoming_event = room_third_party_invite(charlie());

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IPOWER")).unwrap() = to_pdu_event(
        "IPOWER",
        alice(),
        TimelineEventType::RoomPowerLevels,
        Some(""),
        to_raw_json_value(&json!({
            "users": { alice(): 100 },
            "invite": 50,
        }))
        .unwrap(),
        &["CREATE", "IMA"],
        &["IMA"],
    );

    let auth_events = TestStateMap::new(&init_events);
    let fetch_state = auth_events.fetch_state_fn();

    // Cannot accept `m.room.third_party_invite` if not enough power.
    auth_check(&AuthorizationRules::V6, incoming_event, fetch_state).unwrap_err();
}

#[test]
fn room_third_party_invite_with_enough_power() {
    let _guard = init_subscriber();

    let incoming_event = room_third_party_invite(charlie());

    let init_events = INITIAL_EVENTS();
    let auth_events = TestStateMap::new(&init_events);
    let fetch_state = auth_events.fetch_state_fn();

    // Accept `m.room.third_party_invite` if enough power.
    auth_check(&AuthorizationRules::V6, incoming_event, fetch_state).unwrap();
}

#[test]
fn event_type_not_enough_power() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        charlie(),
        TimelineEventType::RoomMessage,
        None,
        to_raw_json_value(&RoomMessageEventContent::text_plain("Hi!")).unwrap(),
        &["CREATE", "IMA", "IPOWER"],
        &["IPOWER"],
    );

    let mut init_events = INITIAL_EVENTS();
    *init_events.get_mut(&event_id("IPOWER")).unwrap() = to_pdu_event(
        "IPOWER",
        alice(),
        TimelineEventType::RoomPowerLevels,
        Some(""),
        to_raw_json_value(&json!({
            "users": { alice(): 100 },
            "events": {
                "m.room.message": "50",
            },
        }))
        .unwrap(),
        &["CREATE", "IMA"],
        &["IMA"],
    );

    let auth_events = TestStateMap::new(&init_events);
    let fetch_state = auth_events.fetch_state_fn();

    // Cannot send event if not enough power for the event's type.
    auth_check(&AuthorizationRules::V6, incoming_event, fetch_state).unwrap_err();
}

#[test]
fn user_id_state_key_not_sender() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        alice(),
        "dev.ruma.fake_state_event".into(),
        Some(ella().as_str()),
        to_raw_json_value(&json!({})).unwrap(),
        &["IMA", "IPOWER", "CREATE"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = TestStateMap::new(&init_events);
    let fetch_state = auth_events.fetch_state_fn();

    // Cannot send state event with a user ID as a state key that doesn't match the sender.
    auth_check(&AuthorizationRules::V6, incoming_event, fetch_state).unwrap_err();
}

#[test]
fn user_id_state_key_is_sender() {
    let _guard = init_subscriber();

    let incoming_event = to_pdu_event(
        "HELLO",
        alice(),
        "dev.ruma.fake_state_event".into(),
        Some(alice().as_str()),
        to_raw_json_value(&json!({})).unwrap(),
        &["IMA", "IPOWER", "CREATE"],
        &["IPOWER"],
    );

    let init_events = INITIAL_EVENTS();
    let auth_events = TestStateMap::new(&init_events);
    let fetch_state = auth_events.fetch_state_fn();

    // Can send state event with a user ID as a state key that matches the sender.
    auth_check(&AuthorizationRules::V6, incoming_event, fetch_state).unwrap();
}
