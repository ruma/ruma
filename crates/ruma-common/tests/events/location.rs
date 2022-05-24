#![cfg(feature = "unstable-msc3488")]

use assert_matches::assert_matches;
use assign::assign;
use js_int::uint;
use ruma_common::{
    event_id,
    events::{
        location::{
            AssetContent, AssetType, LocationContent, LocationEventContent, ZoomLevel,
            ZoomLevelError,
        },
        message::MessageContent,
        room::message::{
            InReplyTo, LocationMessageEventContent, MessageType, Relation, RoomMessageEventContent,
        },
        AnyMessageLikeEvent, MessageLikeEvent, MessageLikeUnsigned, OriginalMessageLikeEvent,
    },
    room_id, user_id, MilliSecondsSinceUnixEpoch,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn plain_content_serialization() {
    let event_content = LocationEventContent::plain(
        "Alice was at geo:51.5008,0.1247;u=35",
        LocationContent::new("geo:51.5008,0.1247;u=35".to_owned()),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": "Alice was at geo:51.5008,0.1247;u=35",
            "m.location": {
                "uri": "geo:51.5008,0.1247;u=35",
            },
        })
    );
}

#[test]
fn event_serialization() {
    let event = OriginalMessageLikeEvent {
        content: assign!(
            LocationEventContent::with_message(
                MessageContent::html(
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
                    in_reply_to: InReplyTo::new(event_id!("$replyevent:example.com").to_owned()),
                }),
            }
        ),
        event_id: event_id!("$event:notareal.hs").to_owned(),
        sender: user_id!("@user:notareal.hs").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(134_829_848)),
        room_id: room_id!("!roomid:notareal.hs").to_owned(),
        unsigned: MessageLikeUnsigned::default(),
    };

    assert_eq!(
        to_json_value(&event).unwrap(),
        json!({
            "content": {
                "org.matrix.msc1767.message": [
                    {
                        "body": "Alice was at <strong>geo:51.5008,0.1247;u=35</strong> as of <em>Sat Nov 13 18:50:58 2021</em>",
                        "mimetype": "text/html",
                    },
                    {
                        "body": "Alice was at geo:51.5008,0.1247;u=35 as of Sat Nov 13 18:50:58 2021",
                        "mimetype": "text/plain",
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
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:notareal.hs",
            "type": "m.location",
        })
    );
}

#[test]
fn plain_content_deserialization() {
    let json_data = json!({
        "m.text": "Alice was at geo:51.5008,0.1247;u=35",
        "m.location": {
            "uri": "geo:51.5008,0.1247;u=35",
        },
    });

    assert_matches!(
        from_json_value::<LocationEventContent>(json_data)
            .unwrap(),
        LocationEventContent {
            message,
            location: LocationContent {
                uri,
                description: None,
                zoom_level: None,
                ..
            },
            asset: AssetContent {
                type_: AssetType::Self_,
                ..
            },
            ts: None,
            ..
        }
        if message.find_plain() == Some("Alice was at geo:51.5008,0.1247;u=35")
            && message.find_html().is_none()
            && uri == "geo:51.5008,0.1247;u=35"
    );
}

#[test]
fn zoomlevel_deserialization_pass() {
    let json_data = json!({
        "uri": "geo:51.5008,0.1247;u=35",
        "zoom_level": 16,
    });

    assert_matches!(
        from_json_value::<LocationContent>(json_data).unwrap(),
        LocationContent {
            zoom_level: Some(zoom_level),
            ..
        } if zoom_level.get() == uint!(16)
    );
}

#[test]
fn zoomlevel_deserialization_too_high() {
    let json_data = json!({
        "uri": "geo:51.5008,0.1247;u=35",
        "zoom_level": 30,
    });

    assert_matches!(
        from_json_value::<LocationContent>(json_data),
        Err(err)
            if err.is_data()
            && format!("{}", err) == format!("{}", ZoomLevelError::TooHigh)
    );
}

#[test]
fn message_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc1767.message": [
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

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::Location(MessageLikeEvent::Original(OriginalMessageLikeEvent {
            content: LocationEventContent {
                message,
                location: LocationContent {
                    uri,
                    description: Some(description),
                    zoom_level: Some(zoom_level),
                    ..
                },
                asset: AssetContent {
                    type_: AssetType::Self_,
                    ..
                },
                ts: Some(ts),
                ..
            },
            event_id,
            origin_server_ts,
            room_id,
            sender,
            unsigned
        })) if event_id == event_id!("$event:notareal.hs")
            && message.find_plain() == Some("Alice was at geo:51.5008,0.1247;u=35 as of Sat Nov 13 18:50:58 2021")
            && message.find_html().is_none()
            && uri == "geo:51.5008,0.1247;u=35"
            && description == "Alice's whereabouts"
            && zoom_level.get() == uint!(4)
            && ts == MilliSecondsSinceUnixEpoch(uint!(1_636_829_458))
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(134_829_848))
            && room_id == room_id!("!roomid:notareal.hs")
            && sender == user_id!("@user:notareal.hs")
            && unsigned.is_empty()
    );
}

#[test]
fn room_message_serialization() {
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
        })
    );
}

#[test]
fn room_message_stable_deserialization() {
    let json_data = json!({
        "body": "Alice was at geo:51.5008,0.1247;u=35",
        "geo_uri": "geo:51.5008,0.1247;u=35",
        "msgtype": "m.location",
        "m.text": "Alice was at geo:51.5008,0.1247;u=35",
        "m.location": {
            "uri": "geo:51.5008,0.1247;u=35",
        },
    });

    let event_content = from_json_value::<RoomMessageEventContent>(json_data).unwrap();
    assert_matches!(event_content.msgtype, MessageType::Location(_));
    if let MessageType::Location(content) = event_content.msgtype {
        assert_eq!(content.body, "Alice was at geo:51.5008,0.1247;u=35");
        assert_eq!(content.geo_uri, "geo:51.5008,0.1247;u=35");
        let message = content.message.unwrap();
        assert_eq!(message.len(), 1);
        assert_eq!(message[0].body, "Alice was at geo:51.5008,0.1247;u=35");
        assert_eq!(content.location.unwrap().uri, "geo:51.5008,0.1247;u=35");
    }
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
    });

    let event_content = from_json_value::<RoomMessageEventContent>(json_data).unwrap();
    assert_matches!(event_content.msgtype, MessageType::Location(_));
    if let MessageType::Location(content) = event_content.msgtype {
        assert_eq!(content.body, "Alice was at geo:51.5008,0.1247;u=35");
        assert_eq!(content.geo_uri, "geo:51.5008,0.1247;u=35");
        let message = content.message.unwrap();
        assert_eq!(message.len(), 1);
        assert_eq!(message[0].body, "Alice was at geo:51.5008,0.1247;u=35");
        assert_eq!(content.location.unwrap().uri, "geo:51.5008,0.1247;u=35");
    }
}
