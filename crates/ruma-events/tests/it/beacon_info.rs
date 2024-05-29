#![cfg(all(feature = "unstable-msc3489", feature = "unstable-msc3488"))]

use std::time::Duration;

use js_int::uint;

use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events::{location::AssetType, room::beacon_info::BeaconInfoEventContent};

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
    serde_json::json!({
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

    assert_eq!(event_content.is_live(), true);
}

#[test]
fn beacon_info_is_not_live() {
    let duration = Some(Duration::from_nanos(1));
    let event_content = get_beacon_info_event_content(duration, None);

    assert_eq!(event_content.is_live(), false);
}

#[test]
fn beacon_info_stop_event() {
    let ts = Some(MilliSecondsSinceUnixEpoch(1_636_829_458_u64.try_into().unwrap()));

    let mut event_content = get_beacon_info_event_content(None, ts);

    event_content.stop();

    assert_eq!(
        serde_json::to_value(&event_content).unwrap(),
        serde_json::json!({
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
        serde_json::to_value(&event_content).unwrap(),
        serde_json::json!({
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
        serde_json::to_value(&event_content).unwrap(),
        serde_json::json!({
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
