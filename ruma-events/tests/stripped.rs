use std::convert::TryFrom;

use js_int::uint;
use ruma_events::{
    room::{join_rules::JoinRule, topic::TopicEventContent},
    AnyStateEventContent, AnyStrippedStateEventStub, StrippedStateEventStub,
};
use ruma_identifiers::UserId;
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn serialize_stripped_state_event_any_content() {
    let event = StrippedStateEventStub {
        content: AnyStateEventContent::RoomTopic(TopicEventContent {
            topic: "Testing room".to_string(),
        }),
        state_key: "".to_string(),
        sender: UserId::try_from("@example:localhost").unwrap(),
    };

    let json_data = json!({
        "content": {
            "topic": "Testing room"
        },
        "type": "m.room.topic",
        "state_key": "",
        "sender": "@example:localhost"
    });

    assert_eq!(to_json_value(&event).unwrap(), json_data);
}

#[test]
fn serialize_stripped_state_event_any_event() {
    let event = AnyStrippedStateEventStub::RoomTopic(StrippedStateEventStub {
        content: TopicEventContent { topic: "Testing room".to_string() },
        state_key: "".to_string(),
        sender: UserId::try_from("@example:localhost").unwrap(),
    });

    let json_data = json!({
        "content": {
            "topic": "Testing room"
        },
        "type": "m.room.topic",
        "state_key": "",
        "sender": "@example:localhost"
    });

    assert_eq!(to_json_value(&event).unwrap(), json_data);
}

#[test]
fn deserialize_stripped_state_events() {
    let name_event = json!({
        "type": "m.room.name",
        "state_key": "",
        "sender": "@example:localhost",
        "content": { "name": "Ruma" }
    });

    let join_rules_event = json!({
        "type": "m.room.join_rules",
        "state_key": "",
        "sender": "@example:localhost",
        "content": { "join_rule": "public" }
    });

    let avatar_event = json!({
        "type": "m.room.avatar",
        "state_key": "",
        "sender": "@example:localhost",
        "content": {
            "info": {
                "h": 128,
                "w": 128,
                "mimetype": "image/jpeg",
                "size": 1024,
                "thumbnail_info": {
                    "h": 16,
                    "w": 16,
                    "mimetype": "image/jpeg",
                    "size": 32
                },
                "thumbnail_url": "https://example.com/image-thumbnail.jpg"
            },
            "thumbnail_info": {
                "h": 16,
                "w": 16,
                "mimetype": "image/jpeg",
                "size": 32
            },
            "thumbnail_url": "https://example.com/image-thumbnail.jpg",
            "url": "https://example.com/image.jpg"
        }
    });

    let event = from_json_value::<AnyStrippedStateEventStub>(name_event).unwrap();
    match event {
        AnyStrippedStateEventStub::RoomName(event) => {
            assert_eq!(event.content.name(), Some("Ruma"));
            assert_eq!(event.state_key, "");
            assert_eq!(event.sender.to_string(), "@example:localhost");
        }
        _ => unreachable!(),
    }

    let event = from_json_value::<AnyStrippedStateEventStub>(join_rules_event).unwrap();
    match event {
        AnyStrippedStateEventStub::RoomJoinRules(event) => {
            assert_eq!(event.content.join_rule, JoinRule::Public);
            assert_eq!(event.state_key, "");
            assert_eq!(event.sender.to_string(), "@example:localhost");
        }
        _ => unreachable!(),
    }

    let event = from_json_value::<AnyStrippedStateEventStub>(avatar_event).unwrap();
    match event {
        AnyStrippedStateEventStub::RoomAvatar(event) => {
            let image_info = event.content.info.unwrap();

            assert_eq!(image_info.height.unwrap(), uint!(128));
            assert_eq!(image_info.width.unwrap(), uint!(128));
            assert_eq!(image_info.mimetype.unwrap(), "image/jpeg");
            assert_eq!(image_info.size.unwrap(), uint!(1024));
            assert_eq!(image_info.thumbnail_info.unwrap().size.unwrap(), uint!(32));
            assert_eq!(event.content.url, "https://example.com/image.jpg");
            assert_eq!(event.state_key, "");
            assert_eq!(event.sender.to_string(), "@example:localhost");
        }
        _ => unreachable!(),
    }
}
