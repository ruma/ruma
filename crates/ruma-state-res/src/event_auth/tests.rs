use ruma_common::user_id;
use ruma_events::TimelineEventType;
use serde_json::{json, value::to_raw_value as to_raw_json_value};

mod room_power_levels;

use super::check_room_create;
use crate::{
    test_utils::{alice, to_init_pdu_event, to_pdu_event},
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
