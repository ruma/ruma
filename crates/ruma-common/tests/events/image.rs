#![cfg(feature = "unstable-msc3552")]

use assert_matches::assert_matches;
use assign::assign;
use js_int::uint;
use ruma_common::{
    event_id,
    events::{
        file::{EncryptedContentInit, FileContent, FileContentInfo},
        image::{
            ImageContent, ImageEventContent, ThumbnailContent, ThumbnailFileContent,
            ThumbnailFileContentInfo,
        },
        message::MessageContent,
        room::{
            message::{
                ImageMessageEventContent, InReplyTo, MessageType, Relation, RoomMessageEventContent,
            },
            JsonWebKeyInit, MediaSource,
        },
        AnyMessageLikeEvent, MessageLikeEvent, MessageLikeUnsigned, OriginalMessageLikeEvent,
    },
    mxc_uri, room_id,
    serde::Base64,
    user_id, MilliSecondsSinceUnixEpoch,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn plain_content_serialization() {
    let event_content = ImageEventContent::plain(
        "Upload: my_image.jpg",
        FileContent::plain(mxc_uri!("mxc://notareal.hs/abcdef").to_owned(), None),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": "Upload: my_image.jpg",
            "m.file": {
                "url": "mxc://notareal.hs/abcdef",
            },
            "m.image": {}
        })
    );
}

#[test]
fn encrypted_content_serialization() {
    let event_content = ImageEventContent::plain(
        "Upload: my_image.jpg",
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
            "org.matrix.msc1767.text": "Upload: my_image.jpg",
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
            "m.image": {}
        })
    );
}

#[test]
fn image_event_serialization() {
    let event = OriginalMessageLikeEvent {
        content: assign!(
            ImageEventContent::with_message(
                MessageContent::html(
                    "Upload: my_house.jpg",
                    "Upload: <strong>my_house.jpg</strong>",
                ),
                FileContent::plain(
                    mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
                    Some(Box::new(assign!(
                        FileContentInfo::new(),
                        {
                            name: Some("my_house.jpg".to_owned()),
                            mimetype: Some("image/jpeg".to_owned()),
                            size: Some(uint!(897_774)),
                        }
                    ))),
                )
            ),
            {
                image: Box::new(ImageContent::with_size(uint!(1920), uint!(1080))),
                thumbnail: vec![ThumbnailContent::new(
                    ThumbnailFileContent::plain(
                        mxc_uri!("mxc://notareal.hs/thumbnail").to_owned(),
                        Some(Box::new(assign!(ThumbnailFileContentInfo::new(), {
                            mimetype: Some("image/jpeg".to_owned()),
                            size: Some(uint!(334_593)),
                        })))
                    ),
                    None
                )],
                caption: Some(MessageContent::plain("This is my house")),
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
                    { "body": "Upload: <strong>my_house.jpg</strong>", "mimetype": "text/html"},
                    { "body": "Upload: my_house.jpg", "mimetype": "text/plain"},
                ],
                "m.file": {
                    "url": "mxc://notareal.hs/abcdef",
                    "name": "my_house.jpg",
                    "mimetype": "image/jpeg",
                    "size": 897_774,
                },
                "m.image": {
                    "width": 1920,
                    "height": 1080,
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
                        "body": "This is my house",
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
            "type": "m.image",
        })
    );
}

#[test]
fn plain_content_deserialization() {
    let json_data = json!({
        "m.text": "Upload: my_cat.png",
        "m.file": {
            "url": "mxc://notareal.hs/abcdef",
        },
        "m.image": {
            "width": 668,
        },
        "m.caption": [
            {
                "body": "Look at my cat!",
            }
        ]
    });

    let content = from_json_value::<ImageEventContent>(json_data).unwrap();
    assert_eq!(content.message.find_plain(), Some("Upload: my_cat.png"));
    assert_eq!(content.message.find_html(), None);
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
    assert_matches!(content.file.encryption_info, None);
    assert_eq!(content.image.width, Some(uint!(668)));
    assert_eq!(content.image.height, None);
    assert!(content.thumbnail.is_empty());
    let caption = content.caption.unwrap();
    assert_eq!(caption.find_plain(), Some("Look at my cat!"));
}

#[test]
fn encrypted_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": "Upload: my_cat.png",
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
        "m.image": {},
        "m.thumbnail": [
            {
                "url": "mxc://notareal.hs/thumbnail",
            }
        ]
    });

    let content = from_json_value::<ImageEventContent>(json_data).unwrap();
    assert_eq!(content.message.find_plain(), Some("Upload: my_cat.png"));
    assert_eq!(content.message.find_html(), None);
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
    assert!(content.file.encryption_info.is_some());
    assert_eq!(content.image.width, None);
    assert_eq!(content.image.height, None);
    assert_eq!(content.thumbnail.len(), 1);
    assert_eq!(content.thumbnail[0].file.url, "mxc://notareal.hs/thumbnail");
    assert_matches!(content.caption, None);
}

#[test]
fn message_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc1767.text": "Upload: my_gnome.webp",
            "m.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "my_gnome.webp",
                "mimetype": "image/webp",
                "size": 123_774,
            },
            "m.image": {
                "width": 1300,
                "height": 837,
            }
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.image",
    });

    let message_event = assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::Image(MessageLikeEvent::Original(message_event))) => message_event
    );

    assert_eq!(message_event.event_id, "$event:notareal.hs");
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(134_829_848)));
    assert_eq!(message_event.room_id, "!roomid:notareal.hs");
    assert_eq!(message_event.sender, "@user:notareal.hs");
    assert!(message_event.unsigned.is_empty());
    let content = message_event.content;
    assert_eq!(content.message.find_plain(), Some("Upload: my_gnome.webp"));
    assert_eq!(content.message.find_html(), None);
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
    let info = content.file.info.unwrap();
    assert_eq!(info.name.as_deref(), Some("my_gnome.webp"));
    assert_eq!(info.mimetype.as_deref(), Some("image/webp"));
    assert_eq!(info.size, Some(uint!(123_774)));
    assert_eq!(content.image.width, Some(uint!(1300)));
    assert_eq!(content.image.height, Some(uint!(837)));
    assert!(content.thumbnail.is_empty());
}

#[test]
fn room_message_serialization() {
    let message_event_content =
        RoomMessageEventContent::new(MessageType::Image(ImageMessageEventContent::plain(
            "Upload: my_image.jpg".to_owned(),
            mxc_uri!("mxc://notareal.hs/file").to_owned(),
            None,
        )));

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "Upload: my_image.jpg",
            "url": "mxc://notareal.hs/file",
            "msgtype": "m.image",
            "org.matrix.msc1767.text": "Upload: my_image.jpg",
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/file",
            },
            "org.matrix.msc1767.image": {},
        })
    );
}

#[test]
fn room_message_stable_deserialization() {
    let json_data = json!({
        "body": "Upload: my_image.jpg",
        "url": "mxc://notareal.hs/file",
        "msgtype": "m.image",
        "m.text": "Upload: my_image.jpg",
        "m.file": {
            "url": "mxc://notareal.hs/file",
        },
        "m.image": {},
    });

    let event_content = from_json_value::<RoomMessageEventContent>(json_data).unwrap();
    let content = assert_matches!(event_content.msgtype, MessageType::Image(content) => content);
    assert_eq!(content.body, "Upload: my_image.jpg");
    let url = assert_matches!(content.source, MediaSource::Plain(url) => url);
    assert_eq!(url, "mxc://notareal.hs/file");
    let message = content.message.unwrap();
    assert_eq!(message.len(), 1);
    assert_eq!(message[0].body, "Upload: my_image.jpg");
    let file = content.file.unwrap();
    assert_eq!(file.url, "mxc://notareal.hs/file");
    assert!(!file.is_encrypted());
}

#[test]
fn room_message_unstable_deserialization() {
    let json_data = json!({
        "body": "Upload: my_image.jpg",
        "url": "mxc://notareal.hs/file",
        "msgtype": "m.image",
        "org.matrix.msc1767.text": "Upload: my_image.jpg",
        "org.matrix.msc1767.file": {
            "url": "mxc://notareal.hs/file",
        },
        "org.matrix.msc1767.image": {},
    });

    let event_content = from_json_value::<RoomMessageEventContent>(json_data).unwrap();
    let content = assert_matches!(event_content.msgtype, MessageType::Image(content) => content);
    assert_eq!(content.body, "Upload: my_image.jpg");
    let url = assert_matches!(content.source, MediaSource::Plain(url) => url);
    assert_eq!(url, "mxc://notareal.hs/file");
    let message = content.message.unwrap();
    assert_eq!(message.len(), 1);
    assert_eq!(message[0].body, "Upload: my_image.jpg");
    let file = content.file.unwrap();
    assert_eq!(file.url, "mxc://notareal.hs/file");
    assert!(!file.is_encrypted());
}
