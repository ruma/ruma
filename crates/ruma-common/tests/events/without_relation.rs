use assert_matches::assert_matches;
use ruma_common::{
    event_id,
    events::room::message::{
        InReplyTo, MessageType, Relation, RoomMessageEventContent,
        RoomMessageEventContentWithoutRelation,
    },
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn serialize_room_message_content_without_relation() {
    let mut content = RoomMessageEventContent::text_plain("Hello, world!");
    content.relates_to =
        Some(Relation::Reply { in_reply_to: InReplyTo::new(event_id!("$eventId").to_owned()) });
    let without_relation = RoomMessageEventContentWithoutRelation::from(content);

    #[cfg(not(feature = "unstable-msc3246"))]
    assert_eq!(
        to_json_value(&without_relation).unwrap(),
        json!({
            "body": "Hello, world!",
            "msgtype": "m.text",
        })
    );

    #[cfg(feature = "unstable-msc3246")]
    assert_eq!(
        to_json_value(&without_relation).unwrap(),
        json!({
            "body": "Hello, world!",
            "msgtype": "m.text",
            "org.matrix.msc1767.text": "Hello, world!",
        })
    );
}

#[test]
fn deserialize_room_message_content_without_relation() {
    let json_data = json!({
        "body": "Hello, world!",
        "msgtype": "m.text",
    });

    let text = assert_matches!(
        from_json_value::<RoomMessageEventContentWithoutRelation>(json_data),
        Ok(RoomMessageEventContentWithoutRelation::Text(text)) => text
    );
    assert_eq!(text.body, "Hello, world!");
}

#[test]
fn convert_room_message_content_without_relation_to_full() {
    let mut content = RoomMessageEventContent::text_plain("Hello, world!");
    content.relates_to =
        Some(Relation::Reply { in_reply_to: InReplyTo::new(event_id!("$eventId").to_owned()) });
    let new_content =
        RoomMessageEventContent::from(RoomMessageEventContentWithoutRelation::from(content));

    let (text, relates_to) = assert_matches!(
        new_content,
        RoomMessageEventContent {
            msgtype: MessageType::Text(text),
            relates_to,
            ..
        } => (text, relates_to)
    );
    assert_eq!(text.body, "Hello, world!");
    assert_matches!(relates_to, None);
}
