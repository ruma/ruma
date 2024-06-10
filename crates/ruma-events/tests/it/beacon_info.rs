#![cfg(feature = "unstable-msc3489")]

use std::time::Duration;

use assert_matches2::assert_matches;
use js_int::uint;
use ruma_common::{event_id, room_id, serde::CanBeEmpty, user_id, MilliSecondsSinceUnixEpoch};
use ruma_events::{
    beacon_info::BeaconInfoEventContent, location::AssetType, AnyStateEvent, StateEvent,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

fn get_beacon_info_event_content(
    duration: Option<Duration>,
    ts: Option<MilliSecondsSinceUnixEpoch>,
) -> BeaconInfoEventContent {
    let description = Some("Kylie's live location".to_owned());
    let duration_or = duration.unwrap_or(Duration::from_secs(60));
    let ts_or = Some(ts.unwrap_or(MilliSecondsSinceUnixEpoch::now()));

    BeaconInfoEventContent::new(description, duration_or, true, ts_or)
}

fn get_beacon_info_json() -> serde_json::Value {
    json!({
        "org.matrix.msc3488.ts": 1_636_829_458,
        "org.matrix.msc3488.asset": {
            "type": "m.self"
        },
        "timeout": 60_000,
        "description": "Kylie's live location",
        "live": true
    })
}

#[test]
fn beacon_info_is_live() {
    let event_content = get_beacon_info_event_content(None, None);

    assert!(event_content.is_live());
}

#[test]
fn beacon_info_is_not_live() {
    let duration = Some(Duration::from_nanos(1));
    let event_content = get_beacon_info_event_content(duration, None);

    assert!(!event_content.is_live());
}

#[test]
fn beacon_info_stop_event() {
    let ts = Some(MilliSecondsSinceUnixEpoch(1_636_829_458_u64.try_into().unwrap()));

    let mut event_content = get_beacon_info_event_content(None, ts);

    event_content.stop();

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc3488.ts": 1_636_829_458,
            "org.matrix.msc3488.asset": {
                "type": "m.self"
            },
            "timeout": 60_000,
            "description": "Kylie's live location",
            "live": false
        })
    );
}

#[test]
fn beacon_info_start_event() {
    let ts = Some(MilliSecondsSinceUnixEpoch(1_636_829_458_u64.try_into().unwrap()));

    let mut event_content = BeaconInfoEventContent::new(
        Some("Kylie's live location".to_owned()),
        Duration::from_secs(60),
        false,
        ts,
    );

    event_content.start();

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc3488.ts": 1_636_829_458,
            "org.matrix.msc3488.asset": {
                "type": "m.self"
            },
            "timeout": 60_000,
            "description": "Kylie's live location",
            "live": true
        })
    );
}

#[test]
fn beacon_info_start_event_content_serialization() {
    let ts = Some(MilliSecondsSinceUnixEpoch(1_636_829_458_u64.try_into().unwrap()));

    let event_content = get_beacon_info_event_content(None, ts);

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc3488.ts": 1_636_829_458,
            "org.matrix.msc3488.asset": {
                "type": "m.self"
            },
            "timeout": 60_000,
            "description": "Kylie's live location",
            "live": true
        })
    );
}

#[test]
fn beacon_info_start_event_content_deserialization() {
    let json_data = get_beacon_info_json();

    let event_content: BeaconInfoEventContent = serde_json::from_value(json_data).unwrap();

    assert_eq!(event_content.description, Some("Kylie's live location".to_owned()));
    assert!(event_content.live);
    assert_eq!(event_content.ts, MilliSecondsSinceUnixEpoch(uint!(1_636_829_458)));
    assert_eq!(event_content.timeout, Duration::from_secs(60));
    assert_eq!(event_content.asset.type_, AssetType::Self_);
}

#[test]
fn state_event_deserialization() {
    let json_data = json!({
        "content": get_beacon_info_json(),
        "event_id": "$beacon_event_id:example.com",
        "origin_server_ts": 1_636_829_458,
        "room_id": "!roomid:example.com",
        "type": "org.matrix.msc3672.beacon_info",
        "sender": "@example:example.com",
        "state_key": "@example:example.com"
    });

    let event = from_json_value::<AnyStateEvent>(json_data).unwrap();

    assert_matches!(event, AnyStateEvent::BeaconInfo(StateEvent::Original(ev)));

    assert_eq!(ev.content.description, Some("Kylie's live location".to_owned()));
    assert_eq!(ev.content.ts, MilliSecondsSinceUnixEpoch(uint!(1_636_829_458)));
    assert_eq!(ev.content.timeout, Duration::from_secs(60));
    assert_eq!(ev.content.asset.type_, AssetType::Self_);
    assert!(ev.content.live);

    assert_eq!(ev.event_id, event_id!("$beacon_event_id:example.com"));
    assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1_636_829_458)));
    assert_eq!(ev.room_id, room_id!("!roomid:example.com"));
    assert_eq!(ev.sender, user_id!("@example:example.com"));
    assert_eq!(ev.state_key, "@example:example.com");
    assert!(ev.unsigned.is_empty());
}
