#![cfg(feature = "unstable-msc3553")]

use std::time::Duration;

use assign::assign;
use js_int::uint;
use matches::assert_matches;
use ruma_common::{
    event_id,
    events::{
        file::{EncryptedContentInit, FileContent, FileContentInfo},
        image::{
            Captions, ThumbnailContent, ThumbnailFileContent, ThumbnailFileContentInfo, Thumbnails,
        },
        message::MessageContent,
        room::{
            message::{InReplyTo, Relation},
            JsonWebKeyInit,
        },
        video::{VideoContent, VideoEventContent},
        AnyMessageLikeEvent, MessageLikeEvent, MessageLikeUnsigned,
    },
    mxc_uri, room_id,
    serde::Base64,
    user_id, MilliSecondsSinceUnixEpoch,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn plain_content_serialization() {
    let event_content = VideoEventContent::plain(
        "Upload: my_video.webm",
        FileContent::plain(mxc_uri!("mxc://notareal.hs/abcdef").to_owned(), None),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": "Upload: my_video.webm",
            "m.file": {
                "url": "mxc://notareal.hs/abcdef",
            },
            "m.video": {}
        })
    );
}

#[test]
fn encrypted_content_serialization() {
    let event_content = VideoEventContent::plain(
        "Upload: my_video.webm",
        FileContent::encrypted(
            mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
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
            None,
        ),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": "Upload: my_video.webm",
            "m.file": {
                "url": "mxc://notareal.hs/abcdef",
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
    let event = MessageLikeEvent {
        content: assign!(
            VideoEventContent::with_message(
                MessageContent::html(
                    "Upload: my_lava_lamp.webm",
                    "Upload: <strong>my_lava_lamp.webm</strong>",
                ),
                FileContent::plain(
                    mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
                    Some(Box::new(assign!(
                        FileContentInfo::new(),
                        {
                            name: Some("my_lava_lamp.webm".to_owned()),
                            mimetype: Some("video/webm".to_owned()),
                            size: Some(uint!(1_897_774)),
                        }
                    ))),
                )
            ),
            {
                video: Box::new(assign!(
                    VideoContent::new(),
                    {
                        width: Some(uint!(1920)),
                        height: Some(uint!(1080)),
                        duration: Some(Duration::from_secs(15)),
                    }
                )),
                thumbnail: Thumbnails::new(&[ThumbnailContent::new(
                    ThumbnailFileContent::plain(
                        mxc_uri!("mxc://notareal.hs/thumbnail").to_owned(),
                        Some(Box::new(assign!(ThumbnailFileContentInfo::new(), {
                            mimetype: Some("image/jpeg".to_owned()),
                            size: Some(uint!(334_593)),
                        })))
                    ),
                    None
                )]),
                caption: Captions::plain("This is my awesome vintage lava lamp"),
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
                    { "body": "Upload: <strong>my_lava_lamp.webm</strong>", "mimetype": "text/html"},
                    { "body": "Upload: my_lava_lamp.webm", "mimetype": "text/plain"},
                ],
                "m.file": {
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
                        "mimetype": "text/plain",
                    }
                ],
                "m.relates_to": {
                    "m.in_reply_to": {
                        "event_id": "$replyevent:example.com"
                    }
                }
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:notareal.hs",
            "type": "m.video",
        })
    );
}

#[test]
fn plain_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": "Video: my_cat.mp4",
        "m.file": {
            "url": "mxc://notareal.hs/abcdef",
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

    assert_matches!(
        from_json_value::<VideoEventContent>(json_data)
            .unwrap(),
        VideoEventContent { message, file, video, thumbnail, caption, .. }
        if message.find_plain() == Some("Video: my_cat.mp4")
            && message.find_html().is_none()
            && file.url == "mxc://notareal.hs/abcdef"
            && video.width.is_none()
            && video.height.is_none()
            && video.duration == Some(Duration::from_millis(5_668))
            && thumbnail.is_empty()
            && caption.find_plain() == Some("Look at my cat!")
    );
}

#[test]
fn encrypted_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": "Upload: my_cat.mp4",
        "m.file": {
            "url": "mxc://notareal.hs/abcdef",
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

    assert_matches!(
        from_json_value::<VideoEventContent>(json_data)
            .unwrap(),
        VideoEventContent { message, file, video, thumbnail, caption, .. }
        if message.find_plain() == Some("Upload: my_cat.mp4")
            && message.find_html().is_none()
            && file.url == "mxc://notareal.hs/abcdef"
            && file.encryption_info.is_some()
            && video.width.is_none()
            && video.height.is_none()
            && video.duration.is_none()
            && thumbnail.thumbnails()[0].file.url == "mxc://notareal.hs/thumbnail"
            && caption.is_empty()
    );
}

#[test]
fn message_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc1767.text": "Upload: my_gnome.webm",
            "m.file": {
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

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::Video(MessageLikeEvent {
            content: VideoEventContent {
                message,
                file: FileContent {
                    url,
                    info: Some(info),
                    ..
                },
                video,
                thumbnail,
                caption,
                ..
            },
            event_id,
            origin_server_ts,
            room_id,
            sender,
            unsigned
        }) if event_id == event_id!("$event:notareal.hs")
            && message.find_plain() == Some("Upload: my_gnome.webm")
            && message.find_html().is_none()
            && url == "mxc://notareal.hs/abcdef"
            && info.name.as_deref() == Some("my_gnome.webm")
            && info.mimetype.as_deref() == Some("video/webm")
            && info.size == Some(uint!(123_774))
            && video.width == Some(uint!(1300))
            && video.height == Some(uint!(837))
            && video.duration.is_none()
            && thumbnail.is_empty()
            && caption.is_empty()
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(134_829_848))
            && room_id == room_id!("!roomid:notareal.hs")
            && sender == user_id!("@user:notareal.hs")
            && unsigned.is_empty()
    );
}
