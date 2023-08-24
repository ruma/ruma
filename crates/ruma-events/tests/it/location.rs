#![cfg(feature = "unstable-msc3488")]

use assert_matches2::assert_matches;
use assign::assign;
use js_int::uint;
use ruma_common::{
    event_id, owned_event_id, room_id, serde::CanBeEmpty, user_id, MilliSecondsSinceUnixEpoch,
};
use ruma_events::{
    location::{AssetType, LocationContent, LocationEventContent, ZoomLevel, ZoomLevelError},
    message::TextContentBlock,
    relation::InReplyTo,
    room::message::{LocationMessageEventContent, MessageType, Relation, RoomMessageEventContent},
    AnyMessageLikeEvent, MessageLikeEvent,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn plain_content_serialization() {
    let event_content = LocationEventContent::with_plain_text(
        "Alice was at geo:51.5008,0.1247;u=35",
        LocationContent::new("geo:51.5008,0.1247;u=35".to_owned()),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "body": "Alice was at geo:51.5008,0.1247;u=35" },
            ],
            "m.location": {
                "uri": "geo:51.5008,0.1247;u=35",
            },
        })
    );
}

#[test]
fn event_serialization() {
    let content = assign!(
        LocationEventContent::new(
            TextContentBlock::html(
                "Alice was at geo:51.5008,0.1247;u=35 as of Sat Nov 13 18:50:58 2021",
                "Alice was at <strong>geo:51.5008,0.1247;u=35</strong> as of <em>Sat Nov 13 18:50:58 2021</em>",
            ),
            assign!(
                LocationContent::new("geo:51.5008,0.1247;u=35".to_owned()),
                {
                    description: Some("Alice's whereabouts".into()),
                    zoom_level: Some(ZoomLevel::new(4).unwrap())
                }
            )
        ),
        {
            ts: Some(MilliSecondsSinceUnixEpoch(uint!(1_636_829_458))),
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo::new(owned_event_id!("$replyevent:example.com")),
            }),
        }
    );

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                {
                    "mimetype": "text/html",
                    "body": "Alice was at <strong>geo:51.5008,0.1247;u=35</strong> as of <em>Sat Nov 13 18:50:58 2021</em>"
                },
                {
                    "body": "Alice was at geo:51.5008,0.1247;u=35 as of Sat Nov 13 18:50:58 2021"
                },
            ],
            "m.location": {
                "uri": "geo:51.5008,0.1247;u=35",
                "description": "Alice's whereabouts",
                "zoom_level": 4,
            },
            "m.ts": 1_636_829_458,
            "m.relates_to": {
                "m.in_reply_to": {
                    "event_id": "$replyevent:example.com",
                },
            },
        })
    );
}

#[test]
fn plain_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": [
            { "body": "Alice was at geo:51.5008,0.1247;u=35" },
        ],
        "m.location": {
            "uri": "geo:51.5008,0.1247;u=35",
        },
    });

    let ev = from_json_value::<LocationEventContent>(json_data).unwrap();

    assert_eq!(ev.text.find_plain(), Some("Alice was at geo:51.5008,0.1247;u=35"));
    assert_eq!(ev.text.find_html(), None);
    assert_eq!(ev.location.uri, "geo:51.5008,0.1247;u=35");
    assert_eq!(ev.location.description, None);
    assert_matches!(ev.location.zoom_level, None);
    assert_eq!(ev.asset.type_, AssetType::Self_);
    assert_eq!(ev.ts, None);
}

#[test]
fn zoomlevel_deserialization_pass() {
    let json_data = json!({
        "uri": "geo:51.5008,0.1247;u=35",
        "zoom_level": 16,
    });

    assert_matches!(
        from_json_value::<LocationContent>(json_data).unwrap(),
        LocationContent { zoom_level: Some(zoom_level), .. }
    );
    assert_eq!(zoom_level.get(), uint!(16));
}

#[test]
fn zoomlevel_deserialization_too_high() {
    let json_data = json!({
        "uri": "geo:51.5008,0.1247;u=35",
        "zoom_level": 30,
    });

    let err = from_json_value::<LocationContent>(json_data).unwrap_err();
    assert!(err.is_data());
    assert_eq!(err.to_string(), ZoomLevelError::TooHigh.to_string());
}

#[test]
fn message_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc1767.text": [
                { "body": "Alice was at geo:51.5008,0.1247;u=35 as of Sat Nov 13 18:50:58 2021" },
            ],
            "m.location": {
                "uri": "geo:51.5008,0.1247;u=35",
                "description": "Alice's whereabouts",
                "zoom_level": 4,
            },
            "m.ts": 1_636_829_458,
            "m.relates_to": {
                "m.in_reply_to": {
                    "event_id": "$replyevent:example.com",
                },
            },
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.location",
    });

    let ev = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();

    assert_matches!(ev, AnyMessageLikeEvent::Location(MessageLikeEvent::Original(ev)));
    assert_eq!(
        ev.content.text.find_plain(),
        Some("Alice was at geo:51.5008,0.1247;u=35 as of Sat Nov 13 18:50:58 2021")
    );
    assert_eq!(ev.content.text.find_html(), None);
    assert_eq!(ev.content.location.uri, "geo:51.5008,0.1247;u=35");
    assert_eq!(ev.content.location.description.as_deref(), Some("Alice's whereabouts"));
    assert_eq!(ev.content.location.zoom_level.unwrap().get(), uint!(4));
    assert_eq!(ev.content.asset.type_, AssetType::Self_);
    assert_eq!(ev.content.ts, Some(MilliSecondsSinceUnixEpoch(uint!(1_636_829_458))));

    assert_eq!(ev.event_id, event_id!("$event:notareal.hs"));
    assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(134_829_848)));
    assert_eq!(ev.room_id, room_id!("!roomid:notareal.hs"));
    assert_eq!(ev.sender, user_id!("@user:notareal.hs"));
    assert!(ev.unsigned.is_empty());
}

#[test]
fn room_message_unstable_deserialization() {
    let json_data = json!({
        "body": "Alice was at geo:51.5008,0.1247;u=35",
        "geo_uri": "geo:51.5008,0.1247;u=35",
        "msgtype": "m.location",
        "org.matrix.msc1767.text": "Alice was at geo:51.5008,0.1247;u=35",
        "org.matrix.msc3488.location": {
            "uri": "geo:51.5008,0.1247;u=35",
        },
        "org.matrix.msc3488.asset": {
            "type": "m.self",
        },
    });

    let event_content = from_json_value::<RoomMessageEventContent>(json_data).unwrap();
    assert_matches!(event_content.msgtype, MessageType::Location(content));

    assert_eq!(content.body, "Alice was at geo:51.5008,0.1247;u=35");
    assert_eq!(content.geo_uri, "geo:51.5008,0.1247;u=35");
    let message = content.message.unwrap();
    assert_eq!(message.len(), 1);
    assert_eq!(message[0].body, "Alice was at geo:51.5008,0.1247;u=35");
    assert_eq!(content.location.unwrap().uri, "geo:51.5008,0.1247;u=35");
    assert_eq!(content.asset.unwrap().type_, AssetType::Self_);
}

#[test]
fn room_message_unstable_serialization() {
    let message_event_content =
        RoomMessageEventContent::new(MessageType::Location(LocationMessageEventContent::new(
            "Alice was at geo:51.5008,0.1247;u=35".to_owned(),
            "geo:51.5008,0.1247;u=35".to_owned(),
        )));
    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "Alice was at geo:51.5008,0.1247;u=35",
            "geo_uri": "geo:51.5008,0.1247;u=35",
            "msgtype": "m.location",
            "org.matrix.msc1767.text": "Alice was at geo:51.5008,0.1247;u=35",
            "org.matrix.msc3488.location": {
                "uri": "geo:51.5008,0.1247;u=35",
            },
            "org.matrix.msc3488.asset": {
                "type": "m.self",
            },
        })
    );
}
