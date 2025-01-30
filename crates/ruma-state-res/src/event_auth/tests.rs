use js_int::int;
use ruma_common::{owned_event_id, user_id};
use ruma_events::{room::redaction::RoomRedactionEventContent, TimelineEventType};
use serde_json::{json, value::to_raw_value as to_raw_json_value};

mod room_power_levels;

use self::room_power_levels::default_room_power_levels;
use super::check_room_create;
use crate::{
    event_auth::check_room_redaction,
    test_utils::{
        alice, charlie, event_id, init_subscriber, room_redaction_pdu_event, to_init_pdu_event,
        to_pdu_event,
    },
    RoomVersion,
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
    assert!(check_room_create(event, &RoomVersion::V1).unwrap());

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
    assert!(check_room_create(event, &RoomVersion::V2).unwrap());

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
    assert!(check_room_create(event, &RoomVersion::V11).unwrap());
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
    assert!(!check_room_create(event, &RoomVersion::V1).unwrap());

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
    assert!(!check_room_create(event, &RoomVersion::V1).unwrap());

    // Room version that is not a string.
    let content = json!({
        "creator": alice(),
        "room_version": 1,
    });
    let event = to_init_pdu_event(
        "CREATE",
        alice(),
        TimelineEventType::RoomCreate,
        Some(""),
        to_raw_json_value(&content).unwrap(),
    );
    assert!(!check_room_create(event, &RoomVersion::V1).unwrap());

    // No creator in v1.
    let content = json!({});
    let event = to_init_pdu_event(
        "CREATE",
        alice(),
        TimelineEventType::RoomCreate,
        Some(""),
        to_raw_json_value(&content).unwrap(),
    );
    assert!(!check_room_create(event, &RoomVersion::V1).unwrap());
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
    assert!(!check_room_redaction(
        incoming_event,
        room_power_levels_event,
        &RoomVersion::V1,
        int!(0)
    )
    .unwrap());
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

    let room_power_levels_event = Some(to_pdu_event(
        "IPOWER",
        alice(),
        TimelineEventType::RoomPowerLevels,
        Some(""),
        to_raw_json_value(&json!({ "users": { alice(): 100, charlie(): 50 } })).unwrap(),
        &["CREATE", "IMA"],
        &["IMA"],
    ));

    // Can redact if redact level is same as user's.
    assert!(check_room_redaction(
        incoming_event,
        room_power_levels_event,
        &RoomVersion::V1,
        int!(50)
    )
    .unwrap());
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
    assert!(check_room_redaction(
        incoming_event,
        room_power_levels_event,
        &RoomVersion::V1,
        int!(0)
    )
    .unwrap());
}
