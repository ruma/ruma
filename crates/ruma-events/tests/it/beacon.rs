#![cfg(feature = "unstable-msc3489")]

use assert_matches2::assert_matches;
use js_int::uint;
use ruma_common::{
    owned_event_id, room_id, serde::CanBeEmpty, user_id, MilliSecondsSinceUnixEpoch,
};
use ruma_events::{
    beacon::BeaconEventContent, relation::Reference, AnyMessageLikeEvent, MessageLikeEvent,
};
use serde_json::{
    from_value as from_json_value, json, to_value as to_json_value, Value as JsonValue,
};

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

    assert_eq!(to_json_value(&event_content).unwrap(), get_beacon_event_content_json());
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
    assert_eq!(event_content.ts, MilliSecondsSinceUnixEpoch(uint!(1_636_829_458)));
}

#[test]
fn message_event_deserialization() {
    let json_data = json!({
        "content": get_beacon_event_content_json(),
        "event_id": "$beacon_event_id:example.com",
        "origin_server_ts": 1_636_829_458,
        "room_id": "!roomid:example.com",
        "type": "org.matrix.msc3672.beacon",
        "sender": "@example:example.com"
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();

    assert_matches!(event, AnyMessageLikeEvent::Beacon(MessageLikeEvent::Original(ev)));
    assert_eq!(ev.content.location.uri, "geo:51.5008,0.1247;u=35");
    assert_eq!(ev.content.ts, MilliSecondsSinceUnixEpoch(uint!(1_636_829_458)));
    assert_matches!(ev.content.relates_to, Reference { event_id, .. });
    assert_eq!(event_id, owned_event_id!("$beacon_info_event_id:example.com"));

    assert_eq!(ev.sender, user_id!("@example:example.com"));
    assert_eq!(ev.room_id, room_id!("!roomid:example.com"));
    assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1_636_829_458)));
    assert!(ev.unsigned.is_empty());
}
