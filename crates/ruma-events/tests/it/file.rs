#![cfg(feature = "unstable-msc3551")]

use assert_matches2::assert_matches;
use js_int::uint;
use ruma_common::{
    mxc_uri, owned_event_id,
    serde::{Base64, CanBeEmpty},
    MilliSecondsSinceUnixEpoch,
};
use ruma_events::{
    file::{EncryptedContentInit, FileEventContent},
    message::TextContentBlock,
    relation::InReplyTo,
    room::{message::Relation, JsonWebKeyInit},
    AnyMessageLikeEvent, MessageLikeEvent,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn plain_content_serialization() {
    let event_content = FileEventContent::plain_with_plain_text(
        "Upload: my_file.txt",
        mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
        "my_file.txt".to_owned(),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "body": "Upload: my_file.txt" },
            ],
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "my_file.txt",
            }
        })
    );
}

#[test]
fn encrypted_content_serialization() {
    let event_content = FileEventContent::encrypted_with_plain_text(
        "Upload: my_file.txt",
        mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
        "my_file.txt".to_owned(),
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
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "body": "Upload: my_file.txt" },
            ],
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "my_file.txt",
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
                "v": "v2",
            }
        })
    );
}

#[test]
fn file_event_serialization() {
    let mut content = FileEventContent::plain(
        TextContentBlock::html("Upload: my_file.txt", "Upload: <strong>my_file.txt</strong>"),
        mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
        "my_file.txt".to_owned(),
    );
    content.file.mimetype = Some("text/plain".to_owned());
    content.file.size = Some(uint!(774));
    content.relates_to = Some(Relation::Reply {
        in_reply_to: InReplyTo::new(owned_event_id!("$replyevent:example.com")),
    });

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "mimetype": "text/html", "body": "Upload: <strong>my_file.txt</strong>" },
                { "body": "Upload: my_file.txt" },
            ],
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "my_file.txt",
                "mimetype": "text/plain",
                "size": 774,
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
            { "body": "Upload: my_file.txt" },
        ],
        "org.matrix.msc1767.file": {
            "url": "mxc://notareal.hs/abcdef",
            "name": "my_file.txt",
        }
    });

    let content = from_json_value::<FileEventContent>(json_data).unwrap();
    assert_eq!(content.text.find_plain(), Some("Upload: my_file.txt"));
    assert_eq!(content.text.find_html(), None);
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
    assert_eq!(content.file.name, "my_file.txt");
}

#[test]
fn encrypted_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": [
            { "body": "Upload: my_file.txt" },
        ],
        "org.matrix.msc1767.file": {
            "url": "mxc://notareal.hs/abcdef",
            "name": "",
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
        }
    });

    let content = from_json_value::<FileEventContent>(json_data).unwrap();
    assert_eq!(content.text.find_plain(), Some("Upload: my_file.txt"));
    assert_eq!(content.text.find_html(), None);
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
    assert_eq!(content.file.name, "");
    assert!(content.file.encryption_info.is_some());
}

#[test]
fn message_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc1767.text": [
                { "body": "Upload: <strong>my_file.txt</strong>", "mimetype": "text/html"},
                { "body": "Upload: my_file.txt", "mimetype": "text/plain"},
            ],
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "my_file.txt",
                "mimetype": "text/plain",
                "size": 774,
            },
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "org.matrix.msc1767.file",
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::File(MessageLikeEvent::Original(message_event)))
    );
    assert_eq!(message_event.event_id, "$event:notareal.hs");
    let content = message_event.content;
    assert_eq!(content.text.find_plain(), Some("Upload: my_file.txt"));
    assert_eq!(content.text.find_html(), Some("Upload: <strong>my_file.txt</strong>"));
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
    assert_eq!(content.file.name, "my_file.txt");
    assert_eq!(content.file.mimetype.as_deref(), Some("text/plain"));
    assert_eq!(content.file.size, Some(uint!(774)));
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(134_829_848)));
    assert_eq!(message_event.room_id, "!roomid:notareal.hs");
    assert_eq!(message_event.sender, "@user:notareal.hs");
    assert!(message_event.unsigned.is_empty());
}
