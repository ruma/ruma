#![cfg(feature = "unstable-msc3553")]

use std::time::Duration;

use assert_matches::assert_matches;
use assign::assign;
use js_int::uint;
use ruma_common::{
    event_id,
    events::{
        file::{EncryptedContentInit, FileContentBlock},
        image::{ThumbnailContent, ThumbnailFileContent, ThumbnailFileContentInfo},
        message::TextContentBlock,
        relation::InReplyTo,
        room::{message::Relation, JsonWebKeyInit},
        video::{VideoContent, VideoEventContent},
        AnyMessageLikeEvent, MessageLikeEvent,
    },
    mxc_uri,
    serde::{Base64, CanBeEmpty},
    MilliSecondsSinceUnixEpoch,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn plain_content_serialization() {
    let event_content = VideoEventContent::plain(
        "Upload: my_video.webm",
        FileContentBlock::plain(
            mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
            "my_video.webm".to_owned(),
        ),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                {"body": "Upload: my_video.webm" },
            ],
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "my_video.webm",
            },
            "m.video": {}
        })
    );
}

#[test]
fn encrypted_content_serialization() {
    let event_content = VideoEventContent::plain(
        "Upload: my_video.webm",
        FileContentBlock::encrypted(
            mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
            "my_video.webm".to_owned(),
            EncryptedContentInit {
                key: JsonWebKeyInit {
                    kty: "oct".to_owned(),
                    key_ops: vec!["encrypt".to_owned(), "decrypt".to_owned()],
                    alg: "A256CTR".to_owned(),
                    k: Base64::parse("TLlG_OpX807zzQuuwv4QZGJ21_u7weemFGYJFszMn9A").unwrap(),
                    ext: true,
                }
                .into(),
                iv: Base64::parse("S22dq3NAX8wAAAAAAAAAAA").unwrap(),
                hashes: [(
                    "sha256".to_owned(),
                    Base64::parse("aWOHudBnDkJ9IwaR1Nd8XKoI7DOrqDTwt6xDPfVGN6Q").unwrap(),
                )]
                .into(),
                v: "v2".to_owned(),
            }
            .into(),
        ),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "body": "Upload: my_video.webm" },
            ],
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "my_video.webm",
                "key": {
                    "kty": "oct",
                    "key_ops": ["encrypt", "decrypt"],
                    "alg": "A256CTR",
                    "k": "TLlG_OpX807zzQuuwv4QZGJ21_u7weemFGYJFszMn9A",
                    "ext": true
                },
                "iv": "S22dq3NAX8wAAAAAAAAAAA",
                "hashes": {
                    "sha256": "aWOHudBnDkJ9IwaR1Nd8XKoI7DOrqDTwt6xDPfVGN6Q"
                },
                "v": "v2"
            },
            "m.video": {}
        })
    );
}

#[test]
fn event_serialization() {
    let mut content = VideoEventContent::new(
        TextContentBlock::html(
            "Upload: my_lava_lamp.webm",
            "Upload: <strong>my_lava_lamp.webm</strong>",
        ),
        FileContentBlock::plain(
            mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
            "my_lava_lamp.webm".to_owned(),
        ),
    );

    content.file.mimetype = Some("video/webm".to_owned());
    content.file.size = Some(uint!(1_897_774));
    content.video = Box::new(assign!(
        VideoContent::new(),
        {
            width: Some(uint!(1920)),
            height: Some(uint!(1080)),
            duration: Some(Duration::from_secs(15)),
        }
    ));
    content.thumbnail = vec![ThumbnailContent::new(
        ThumbnailFileContent::plain(
            mxc_uri!("mxc://notareal.hs/thumbnail").to_owned(),
            Some(Box::new(assign!(ThumbnailFileContentInfo::new(), {
                mimetype: Some("image/jpeg".to_owned()),
                size: Some(uint!(334_593)),
            }))),
        ),
        None,
    )];
    content.caption = TextContentBlock::plain("This is my awesome vintage lava lamp");
    content.relates_to = Some(Relation::Reply {
        in_reply_to: InReplyTo::new(event_id!("$replyevent:example.com").to_owned()),
    });

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "mimetype": "text/html", "body": "Upload: <strong>my_lava_lamp.webm</strong>" },
                { "body": "Upload: my_lava_lamp.webm" },
            ],
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "my_lava_lamp.webm",
                "mimetype": "video/webm",
                "size": 1_897_774,
            },
            "m.video": {
                "width": 1920,
                "height": 1080,
                "duration": 15_000,
            },
            "m.thumbnail": [
                {
                    "url": "mxc://notareal.hs/thumbnail",
                    "mimetype": "image/jpeg",
                    "size": 334_593,
                }
            ],
            "m.caption": [
                {
                    "body": "This is my awesome vintage lava lamp",
                }
            ],
            "m.relates_to": {
                "m.in_reply_to": {
                    "event_id": "$replyevent:example.com"
                }
            }
        })
    );
}

#[test]
fn plain_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": [
            { "body": "Video: my_cat.mp4" },
        ],
        "org.matrix.msc1767.file": {
            "url": "mxc://notareal.hs/abcdef",
            "name": "my_cat.mp4",
        },
        "m.video": {
            "duration": 5_668,
        },
        "m.caption": [
            {
                "body": "Look at my cat!",
            }
        ]
    });

    let content = from_json_value::<VideoEventContent>(json_data).unwrap();
    assert_eq!(content.text.find_plain(), Some("Video: my_cat.mp4"));
    assert_eq!(content.text.find_html(), None);
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
    assert_eq!(content.file.name, "my_cat.mp4");
    assert_matches!(content.file.encryption_info, None);
    assert_eq!(content.video.width, None);
    assert_eq!(content.video.height, None);
    assert_eq!(content.video.duration, Some(Duration::from_millis(5_668)));
    assert_eq!(content.thumbnail.len(), 0);
    assert_eq!(content.caption.find_plain(), Some("Look at my cat!"));
    assert_eq!(content.caption.find_html(), None);
}

#[test]
fn encrypted_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": [
            { "body": "Video: my_cat.mp4" },
        ],
        "org.matrix.msc1767.file": {
            "url": "mxc://notareal.hs/abcdef",
            "name": "my_cat.mp4",
            "key": {
                "kty": "oct",
                "key_ops": ["encrypt", "decrypt"],
                "alg": "A256CTR",
                "k": "TLlG_OpX807zzQuuwv4QZGJ21_u7weemFGYJFszMn9A",
                "ext": true
            },
            "iv": "S22dq3NAX8wAAAAAAAAAAA",
            "hashes": {
                "sha256": "aWOHudBnDkJ9IwaR1Nd8XKoI7DOrqDTwt6xDPfVGN6Q"
            },
            "v": "v2"
        },
        "m.video": {},
        "m.thumbnail": [
            {
                "url": "mxc://notareal.hs/thumbnail",
            }
        ]
    });

    let content = from_json_value::<VideoEventContent>(json_data).unwrap();
    assert_eq!(content.text.find_plain(), Some("Video: my_cat.mp4"));
    assert_eq!(content.text.find_html(), None);
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
    assert_eq!(content.file.name, "my_cat.mp4");
    assert!(content.file.encryption_info.is_some());
    assert_eq!(content.video.width, None);
    assert_eq!(content.video.height, None);
    assert_eq!(content.video.duration, None);
    assert_eq!(content.thumbnail.len(), 1);
    assert_eq!(content.thumbnail[0].file.url, "mxc://notareal.hs/thumbnail");
    assert!(content.caption.is_empty());
}

#[test]
fn message_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc1767.text": [
                { "body": "Upload: my_gnome.webm" },
            ],
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "my_gnome.webm",
                "mimetype": "video/webm",
                "size": 123_774,
            },
            "m.video": {
                "width": 1300,
                "height": 837,
            }
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.video",
    });

    let ev = assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::Video(MessageLikeEvent::Original(ev))) => ev
    );
    assert_eq!(ev.event_id, "$event:notareal.hs");
    assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(134_829_848)));
    assert_eq!(ev.room_id, "!roomid:notareal.hs");
    assert_eq!(ev.sender, "@user:notareal.hs");
    assert!(ev.unsigned.is_empty());

    let content = ev.content;
    assert_eq!(content.text.find_plain(), Some("Upload: my_gnome.webm"));
    assert_eq!(content.text.find_html(), None);
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
    assert_eq!(content.file.name, "my_gnome.webm");
    assert_eq!(content.file.mimetype.as_deref(), Some("video/webm"));
    assert_eq!(content.file.size, Some(uint!(123_774)));
    assert_eq!(content.video.width, Some(uint!(1300)));
    assert_eq!(content.video.height, Some(uint!(837)));
    assert_eq!(content.video.duration, None);
    assert_eq!(content.thumbnail.len(), 0);
}
