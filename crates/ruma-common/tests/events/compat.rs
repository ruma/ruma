#![cfg(feature = "compat")]

use matches::assert_matches;
use ruma_common::events::room::topic::{RoomTopicEvent, RoomTopicEventContent};
use serde_json::{from_value as from_json_value, json};

#[test]
fn deserialize_unsigned_prev_content() {
    let res = from_json_value::<RoomTopicEvent>(json!({
        "content": {
            "topic": "New room topic",
        },
        "event_id": "$143273582443PhrSn:example.org",
        "origin_server_ts": 1_432_735_824_653_u64,
        "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
        "sender": "@example:example.org",
        "state_key": "",
        "type": "m.room.topic",
        "unsigned": {
            "age": 1234,
            "prev_content": {
                "topic": "Old room topic",
            },
        },
    }));

    assert_matches!(
        res,
        Ok(RoomTopicEvent {
            content: RoomTopicEventContent { topic: new_topic, .. },
            prev_content: Some(RoomTopicEventContent { topic: old_topic, .. }),
            ..
        }) if new_topic == "New room topic"
            && old_topic == "Old room topic"
    );
}
