use assert_matches2::assert_matches;
use ruma_common::owned_event_id;
use ruma_events::{
    relation::InReplyTo,
    room::message::{MessageType, Relation, RoomMessageEventContent},
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn serialize_room_message_content_without_relation() {
    let mut content = RoomMessageEventContent::text_plain("Hello, world!");
    content.relates_to =
        Some(Relation::Reply { in_reply_to: InReplyTo::new(owned_event_id!("$eventId")) });
    let without_relation = MessageType::from(content);

    assert_eq!(
        to_json_value(&without_relation).unwrap(),
        json!({
            "body": "Hello, world!",
            "msgtype": "m.text",
        })
    );
}

#[test]
fn deserialize_room_message_content_without_relation() {
    let json_data = json!({
        "body": "Hello, world!",
        "msgtype": "m.text",
    });

    assert_matches!(from_json_value::<MessageType>(json_data), Ok(MessageType::Text(text)));
    assert_eq!(text.body, "Hello, world!");
}

#[test]
fn convert_room_message_content_without_relation_to_full() {
    let mut content = RoomMessageEventContent::text_plain("Hello, world!");
    content.relates_to =
        Some(Relation::Reply { in_reply_to: InReplyTo::new(owned_event_id!("$eventId")) });
    let new_content = RoomMessageEventContent::from(MessageType::from(content));

    assert_matches!(
        new_content,
        RoomMessageEventContent { msgtype: MessageType::Text(text), relates_to, .. }
    );
    assert_eq!(text.body, "Hello, world!");
    assert_matches!(relates_to, None);
}
