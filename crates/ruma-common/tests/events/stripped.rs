use assert_matches::assert_matches;
use js_int::uint;
use ruma_common::{
    events::{
        room::{join_rules::JoinRule, topic::RoomTopicEventContent},
        AnyStrippedStateEvent, EmptyStateKey, StrippedStateEvent,
    },
    mxc_uri, user_id,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn serialize_stripped_state_event_any_content() {
    let event = StrippedStateEvent {
        content: RoomTopicEventContent::new("Testing room".into()),
        state_key: EmptyStateKey,
        sender: user_id!("@example:localhost").to_owned(),
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
                "thumbnail_url": "mxc://example.com/THumbNa1l"
            },
            "thumbnail_info": {
                "h": 16,
                "w": 16,
                "mimetype": "image/jpeg",
                "size": 32
            },
            "thumbnail_url": "mxc://example.com/THumbNa1l",
            "url": "mxc://example.com/iMag3"
        }
    });

    let ev = from_json_value::<AnyStrippedStateEvent>(name_event).unwrap();
    let ev = assert_matches!(ev, AnyStrippedStateEvent::RoomName(ev) => ev);
    assert_eq!(ev.content.name.as_deref(), Some("Ruma"));
    assert_eq!(ev.sender.to_string(), "@example:localhost");

    let ev = from_json_value::<AnyStrippedStateEvent>(join_rules_event).unwrap();
    let ev = assert_matches!(ev,AnyStrippedStateEvent::RoomJoinRules(ev)=>ev );
    assert_eq!(ev.content.join_rule, JoinRule::Public);
    assert_eq!(ev.sender.to_string(), "@example:localhost");

    let ev = from_json_value::<AnyStrippedStateEvent>(avatar_event).unwrap();
    let ev = assert_matches!(ev, AnyStrippedStateEvent::RoomAvatar(ev) => ev);
    assert_eq!(ev.content.url.unwrap(), mxc_uri!("mxc://example.com/iMag3"));
    assert_eq!(ev.sender.to_string(), "@example:localhost");

    let image_info = ev.content.info.unwrap();
    assert_eq!(image_info.height, Some(uint!(128)));
    assert_eq!(image_info.width, Some(uint!(128)));
    assert_eq!(image_info.mimetype.as_deref(), Some("image/jpeg"));
    assert_eq!(image_info.size, Some(uint!(1024)));
    assert_eq!(image_info.thumbnail_info.unwrap().size, Some(uint!(32)));
}
