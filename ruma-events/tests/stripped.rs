use js_int::uint;
use ruma_events::{
    room::{join_rules::JoinRule, topic::TopicEventContent},
    AnyStateEventContent, AnyStrippedStateEvent, StrippedStateEvent,
};
use ruma_identifiers::user_id;
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn serialize_stripped_state_event_any_content() {
    let event = StrippedStateEvent {
        content: AnyStateEventContent::RoomTopic(TopicEventContent {
            topic: "Testing room".into(),
        }),
        state_key: "".into(),
        sender: user_id!("@example:localhost"),
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
    let event = AnyStrippedStateEvent::RoomTopic(StrippedStateEvent {
        content: TopicEventContent { topic: "Testing room".into() },
        state_key: "".into(),
        sender: user_id!("@example:localhost"),
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

    let event = from_json_value::<AnyStrippedStateEvent>(name_event).unwrap();
    match event {
        AnyStrippedStateEvent::RoomName(event) => {
            assert_eq!(event.content.name(), Some("Ruma"));
            assert_eq!(event.state_key, "");
            assert_eq!(event.sender.to_string(), "@example:localhost");
        }
        _ => unreachable!(),
    }

    let event = from_json_value::<AnyStrippedStateEvent>(join_rules_event).unwrap();
    match event {
        AnyStrippedStateEvent::RoomJoinRules(event) => {
            assert_eq!(event.content.join_rule, JoinRule::Public);
            assert_eq!(event.state_key, "");
            assert_eq!(event.sender.to_string(), "@example:localhost");
        }
        _ => unreachable!(),
    }

    let event = from_json_value::<AnyStrippedStateEvent>(avatar_event).unwrap();
    match event {
        AnyStrippedStateEvent::RoomAvatar(event) => {
            let image_info = event.content.info.unwrap();
            let expected_url = "https://example.com/image.jpg";

            #[cfg(feature = "unstable-pre-spec")]
            let expected_url = Some(expected_url.to_owned());

            assert_eq!(image_info.height.unwrap(), uint!(128));
            assert_eq!(image_info.width.unwrap(), uint!(128));
            assert_eq!(image_info.mimetype.unwrap(), "image/jpeg");
            assert_eq!(image_info.size.unwrap(), uint!(1024));
            assert_eq!(image_info.thumbnail_info.unwrap().size.unwrap(), uint!(32));
            assert_eq!(event.content.url, expected_url);
            assert_eq!(event.state_key, "");
            assert_eq!(event.sender.to_string(), "@example:localhost");
        }
        _ => unreachable!(),
    }
}
