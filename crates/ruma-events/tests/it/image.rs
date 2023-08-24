#![cfg(feature = "unstable-msc3552")]

use assert_matches2::assert_matches;
use js_int::uint;
use ruma_common::{
    mxc_uri, owned_event_id,
    serde::{Base64, CanBeEmpty},
    MilliSecondsSinceUnixEpoch,
};
use ruma_events::{
    file::{CaptionContentBlock, EncryptedContentInit, FileContentBlock},
    image::{
        ImageDetailsContentBlock, ImageEventContent, Thumbnail, ThumbnailFileContentBlock,
        ThumbnailImageDetailsContentBlock,
    },
    message::TextContentBlock,
    relation::InReplyTo,
    room::{message::Relation, JsonWebKeyInit},
    AnyMessageLikeEvent, MessageLikeEvent,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn plain_content_serialization() {
    let event_content = ImageEventContent::with_plain_text(
        "Upload: my_image.jpg",
        FileContentBlock::plain(
            mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
            "my_image.jpg".to_owned(),
        ),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "body": "Upload: my_image.jpg" },
            ],
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "my_image.jpg",
            },
        })
    );
}

#[test]
fn encrypted_content_serialization() {
    let event_content = ImageEventContent::with_plain_text(
        "Upload: my_image.jpg",
        FileContentBlock::encrypted(
            mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
            "my_image.jpg".to_owned(),
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
                { "body": "Upload: my_image.jpg" },
            ],
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "my_image.jpg",
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
        })
    );
}

#[test]
fn image_event_serialization() {
    let mut content = ImageEventContent::new(
        TextContentBlock::html("Upload: my_house.jpg", "Upload: <strong>my_house.jpg</strong>"),
        FileContentBlock::plain(
            mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
            "my_house.jpg".to_owned(),
        ),
    );

    content.file.mimetype = Some("image/jpeg".to_owned());
    content.file.size = Some(uint!(897_774));
    content.image_details = Some(ImageDetailsContentBlock::new(uint!(1920), uint!(1080)));
    let mut thumbnail = Thumbnail::new(
        ThumbnailFileContentBlock::plain(
            mxc_uri!("mxc://notareal.hs/thumbnail").to_owned(),
            "image/jpeg".to_owned(),
        ),
        ThumbnailImageDetailsContentBlock::new(uint!(560), uint!(480)),
    );
    thumbnail.file.size = Some(uint!(334_593));
    content.thumbnail = vec![thumbnail].into();
    content.caption = Some(CaptionContentBlock::plain("This is my house"));
    content.relates_to = Some(Relation::Reply {
        in_reply_to: InReplyTo::new(owned_event_id!("$replyevent:example.com")),
    });

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "mimetype": "text/html", "body": "Upload: <strong>my_house.jpg</strong>" },
                { "body": "Upload: my_house.jpg" },
            ],
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "my_house.jpg",
                "mimetype": "image/jpeg",
                "size": 897_774,
            },
            "org.matrix.msc1767.image_details": {
                "width": 1920,
                "height": 1080,
            },
            "org.matrix.msc1767.thumbnail": [
                {
                    "org.matrix.msc1767.file": {
                        "url": "mxc://notareal.hs/thumbnail",
                        "mimetype": "image/jpeg",
                        "size": 334_593,
                    },
                    "org.matrix.msc1767.image_details": {
                        "width": 560,
                        "height": 480,
                    },
                },
            ],
            "org.matrix.msc1767.caption": {
                "org.matrix.msc1767.text": [
                    { "body": "This is my house" },
                ],
            },
            "m.relates_to": {
                "m.in_reply_to": {
                    "event_id": "$replyevent:example.com"
                }
            },
        })
    );
}

#[test]
fn plain_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": [
            { "body": "Upload: my_cat.png" },
        ],
        "org.matrix.msc1767.file": {
            "url": "mxc://notareal.hs/abcdef",
            "name": "my_cat.png",
        },
        "org.matrix.msc1767.image_details": {
            "width": 668,
            "height": 1023,
        },
        "org.matrix.msc1767.caption": {
            "org.matrix.msc1767.text": [
                { "body": "Look at my cat!" },
            ],
        },
    });

    let content = from_json_value::<ImageEventContent>(json_data).unwrap();
    assert_eq!(content.text.find_plain(), Some("Upload: my_cat.png"));
    assert_eq!(content.text.find_html(), None);
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
    assert_eq!(content.file.name, "my_cat.png");
    assert_matches!(content.file.encryption_info, None);
    let image_details = content.image_details.unwrap();
    assert_eq!(image_details.width, uint!(668));
    assert_eq!(image_details.height, uint!(1023));
    assert_eq!(content.thumbnail.len(), 0);
    let caption = content.caption.unwrap();
    assert_eq!(caption.text.len(), 1);
    assert_eq!(caption.text.find_plain(), Some("Look at my cat!"));
}

#[test]
fn encrypted_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": [
            { "body": "Upload: my_cat.png" },
        ],
        "org.matrix.msc1767.file": {
            "url": "mxc://notareal.hs/abcdef",
            "name": "my_cat.png",
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
        "org.matrix.msc1767.thumbnail": [
            {
                "org.matrix.msc1767.file": {
                    "url": "mxc://notareal.hs/thumbnail",
                    "mimetype": "image/png",
                },
                "org.matrix.msc1767.image_details": {
                    "width": 480,
                    "height": 560,
                }
            }
        ]
    });

    let content = from_json_value::<ImageEventContent>(json_data).unwrap();
    assert_eq!(content.text.find_plain(), Some("Upload: my_cat.png"));
    assert_eq!(content.text.find_html(), None);
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
    assert_eq!(content.file.name, "my_cat.png");
    assert!(content.file.encryption_info.is_some());
    assert!(content.image_details.is_none());
    assert_eq!(content.thumbnail.len(), 1);
    let thumbnail = &content.thumbnail[0];
    assert_eq!(thumbnail.file.url, "mxc://notareal.hs/thumbnail");
    assert_eq!(thumbnail.file.mimetype, "image/png");
    assert_eq!(thumbnail.image_details.width, uint!(480));
    assert_eq!(thumbnail.image_details.height, uint!(560));
    assert!(content.caption.is_none());
}

#[test]
fn message_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc1767.text": [
                { "body": "Upload: my_gnome.webp" },
            ],
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "my_gnome.webp",
                "mimetype": "image/webp",
                "size": 123_774,
            },
            "org.matrix.msc1767.image_details": {
                "width": 1300,
                "height": 837,
            }
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "org.matrix.msc1767.image",
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::Image(MessageLikeEvent::Original(message_event)))
    );

    assert_eq!(message_event.event_id, "$event:notareal.hs");
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(134_829_848)));
    assert_eq!(message_event.room_id, "!roomid:notareal.hs");
    assert_eq!(message_event.sender, "@user:notareal.hs");
    assert!(message_event.unsigned.is_empty());
    let content = message_event.content;
    assert_eq!(content.text.find_plain(), Some("Upload: my_gnome.webp"));
    assert_eq!(content.text.find_html(), None);
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
    assert_eq!(content.file.name, "my_gnome.webp");
    assert_eq!(content.file.mimetype.as_deref(), Some("image/webp"));
    assert_eq!(content.file.size, Some(uint!(123_774)));
    let image_details = content.image_details.unwrap();
    assert_eq!(image_details.width, uint!(1300));
    assert_eq!(image_details.height, uint!(837));
    assert_eq!(content.thumbnail.len(), 0);
}
