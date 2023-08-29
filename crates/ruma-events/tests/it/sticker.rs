use assert_matches2::assert_matches;
use assign::assign;
use js_int::{uint, UInt};
use ruma_common::{mxc_uri, serde::CanBeEmpty, MilliSecondsSinceUnixEpoch};
use ruma_events::{
    room::{ImageInfo, MediaSource, ThumbnailInfo},
    sticker::StickerEventContent,
    AnyMessageLikeEvent, MessageLikeEvent,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn content_serialization() {
    let message_event_content = StickerEventContent::new(
        "Upload: my_image.jpg".to_owned(),
        ImageInfo::new(),
        mxc_uri!("mxc://notareal.hs/file").to_owned(),
    );

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "Upload: my_image.jpg",
            "url": "mxc://notareal.hs/file",
            "info": {},
        })
    );
}

#[test]
fn event_serialization() {
    let content = StickerEventContent::new(
        "Hello".into(),
        assign!(ImageInfo::new(), {
            height: UInt::new(423),
            width: UInt::new(1011),
            mimetype: Some("image/png".into()),
            size: UInt::new(84242),
            thumbnail_info: Some(Box::new(assign!(ThumbnailInfo::new(), {
                width: UInt::new(800),
                height: UInt::new(334),
                mimetype: Some("image/png".into()),
                size: UInt::new(82595),
            }))),
            thumbnail_source: Some(MediaSource::Plain(mxc_uri!("mxc://matrix.org/irsns989Rrsn").to_owned())),
        }),
        mxc_uri!("mxc://matrix.org/rnsldl8srs98IRrs").to_owned(),
    );

    let actual = to_json_value(&content).unwrap();

    let expected = json!({
        "body": "Hello",
        "info": {
            "h": 423,
            "mimetype": "image/png",
            "size": 84242,
            "thumbnail_info": {
                "h": 334,
                "mimetype": "image/png",
                "size": 82595,
                "w": 800
            },
            "thumbnail_url": "mxc://matrix.org/irsns989Rrsn",
            "w": 1011
            },
        "url": "mxc://matrix.org/rnsldl8srs98IRrs",
    });
    assert_eq!(actual, expected);
}

#[test]
fn content_deserialization() {
    let json_data = json!({
        "body": "Upload: my_image.jpg",
        "url": "mxc://notareal.hs/file",
        "info": {},
    });

    let content = from_json_value::<StickerEventContent>(json_data).unwrap();
    assert_eq!(content.body, "Upload: my_image.jpg");
    assert_eq!(content.url, "mxc://notareal.hs/file");
}

#[test]
fn event_deserialization() {
    let json_data = json!({
        "content": {
            "body": "Hello",
            "info": {
                "h": 423,
                "mimetype": "image/png",
                "size": 84242,
                "thumbnail_info": {
                  "h": 334,
                  "mimetype": "image/png",
                  "size": 82595,
                  "w": 800
                },
                "thumbnail_url": "mxc://matrix.org/irnsNRS2879",
                "w": 1011
              },
            "url": "mxc://matrix.org/jxPXTKpyydzdHJkdFNZjTZrD"
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "sender": "@carl:example.com",
        "type": "m.sticker"
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::Sticker(MessageLikeEvent::Original(message_event)))
    );

    assert_eq!(message_event.event_id, "$h29iv0s8:example.com");
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(message_event.room_id, "!roomid:room.com");
    assert_eq!(message_event.sender, "@carl:example.com");
    assert!(message_event.unsigned.is_empty());

    let content = message_event.content;
    assert_eq!(content.body, "Hello");
    assert_eq!(content.info.height, Some(uint!(423)));
    assert_eq!(content.info.width, Some(uint!(1011)));
    assert_eq!(content.info.mimetype.as_deref(), Some("image/png"));
    assert_eq!(content.info.size, Some(uint!(84242)));
    assert_eq!(content.url, "mxc://matrix.org/jxPXTKpyydzdHJkdFNZjTZrD");

    assert_matches!(content.info.thumbnail_source, Some(MediaSource::Plain(thumbnail_url)));
    assert_eq!(thumbnail_url, "mxc://matrix.org/irnsNRS2879");
    let thumbnail_info = content.info.thumbnail_info.unwrap();
    assert_eq!(thumbnail_info.width, Some(uint!(800)));
    assert_eq!(thumbnail_info.height, Some(uint!(334)));
    assert_eq!(thumbnail_info.mimetype.as_deref(), Some("image/png"));
    assert_eq!(thumbnail_info.size, Some(uint!(82595)));
}
