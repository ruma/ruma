use std::convert::TryFrom;

use js_int::UInt;
use ruma_events::{
    room::{join_rules::JoinRule, topic::TopicEventContent},
    AnyStateEventContent, AnyStrippedStateEventStub,
};
use ruma_identifiers::UserId;
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn serialize_stripped_state_event() {
    let event = AnyStrippedStateEventStub {
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

    let event = from_json_value::<AnyStrippedStateEventStub>(name_event.clone()).unwrap();
    match event.content {
        AnyStateEventContent::RoomName(content) => {
            assert_eq!(content.name(), Some("Ruma"));
            assert_eq!(event.state_key, "");
            assert_eq!(event.sender.to_string(), "@example:localhost");
        }
        _ => unreachable!(),
    }

    let event = from_json_value::<AnyStrippedStateEventStub>(join_rules_event.clone()).unwrap();
    match event.content {
        AnyStateEventContent::RoomJoinRules(content) => {
            assert_eq!(content.join_rule, JoinRule::Public);
            assert_eq!(event.state_key, "");
            assert_eq!(event.sender.to_string(), "@example:localhost");
        }
        _ => unreachable!(),
    }

    let event = from_json_value::<AnyStrippedStateEventStub>(avatar_event.clone()).unwrap();
    match event.content {
        AnyStateEventContent::RoomAvatar(content) => {
            let image_info = content.info.unwrap();

            assert_eq!(image_info.height.unwrap(), UInt::try_from(128).unwrap());
            assert_eq!(image_info.width.unwrap(), UInt::try_from(128).unwrap());
            assert_eq!(image_info.mimetype.unwrap(), "image/jpeg");
            assert_eq!(image_info.size.unwrap(), UInt::try_from(1024).unwrap());
            assert_eq!(
                image_info.thumbnail_info.unwrap().size.unwrap(),
                UInt::try_from(32).unwrap()
            );
            assert_eq!(content.url, "https://example.com/image.jpg");
            assert_eq!(event.state_key, "");
            assert_eq!(event.sender.to_string(), "@example:localhost");
        }
        _ => unreachable!(),
    }
}
