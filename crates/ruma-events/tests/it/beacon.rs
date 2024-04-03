#![cfg(all(feature = "unstable-msc3489", feature = "unstable-msc3488"))]

use js_int::uint;
use ruma_common::{owned_event_id, MilliSecondsSinceUnixEpoch};
use ruma_events::beacon::BeaconEventContent;
use serde_json::{from_value as from_json_value, json, Value as JsonValue};

fn get_beacon_event_content() -> BeaconEventContent {
    BeaconEventContent::new(
        owned_event_id!("$beacon_info_event_id:example.com"),
        "geo:51.5008,0.1247;u=35".to_owned(),
        Some(MilliSecondsSinceUnixEpoch(uint!(1_636_829_458))),
    )
}

fn get_beacon_event_content_json() -> JsonValue {
    json!({
        "m.relates_to": {
            "rel_type": "m.reference",
            "event_id": "$beacon_info_event_id:example.com"
        },
        "org.matrix.msc3488.location": {
            "uri": "geo:51.5008,0.1247;u=35",
        },
        "org.matrix.msc3488.ts": 1_636_829_458
    })
}

#[test]
fn beacon_event_content_serialization() {
    let event_content = get_beacon_event_content();

    assert_eq!(serde_json::to_value(&event_content).unwrap(), get_beacon_event_content_json());
}

#[test]
fn beacon_event_content_deserialization() {
    let json_data = get_beacon_event_content_json();

    let event_content: BeaconEventContent =
        from_json_value::<BeaconEventContent>(json_data).unwrap();

    assert_eq!(
        event_content.relates_to.event_id,
        owned_event_id!("$beacon_info_event_id:example.com")
    );
    assert_eq!(event_content.location.uri, "geo:51.5008,0.1247;u=35");
}
