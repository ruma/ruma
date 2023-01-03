#![cfg(feature = "unstable-msc3551")]

use assert_matches::assert_matches;
use assign::assign;
use js_int::uint;
use ruma_common::{
    event_id,
    events::{
        file::{EncryptedContentInit, FileContentInfo, FileEventContent},
        message::TextContentBlock,
        relation::InReplyTo,
        room::{message::Relation, JsonWebKeyInit},
        AnyMessageLikeEvent, MessageLikeEvent,
    },
    mxc_uri,
    serde::{Base64, CanBeEmpty},
    MilliSecondsSinceUnixEpoch,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn plain_content_serialization() {
    let event_content = FileEventContent::plain_with_text(
        "Upload: my_file.txt",
        mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
        None,
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "body": "Upload: my_file.txt" },
            ],
            "m.file": {
                "url": "mxc://notareal.hs/abcdef",
            }
        })
    );
}

#[test]
fn encrypted_content_serialization() {
    let event_content = FileEventContent::encrypted_with_text(
        "Upload: my_file.txt",
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
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "body": "Upload: my_file.txt" },
            ],
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
            }
        })
    );
}

#[test]
fn file_event_serialization() {
    let content = assign!(
            FileEventContent::plain(
                TextContentBlock::html(
                    "Upload: my_file.txt",
                    "Upload: <strong>my_file.txt</strong>",
                ),
                mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
                Some(Box::new(assign!(
                    FileContentInfo::new(),
                    {
                        name: Some("my_file.txt".to_owned()),
                        mimetype: Some("text/plain".to_owned()),
                        size: Some(uint!(774)),
                    }
                ))),
            ),
            {
                relates_to: Some(Relation::Reply {
                    in_reply_to: InReplyTo::new(event_id!("$replyevent:example.com").to_owned()),
                }),
            }
        );

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "mimetype": "text/html", "body": "Upload: <strong>my_file.txt</strong>" },
                { "body": "Upload: my_file.txt" },
            ],
            "m.file": {
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
        "m.file": {
            "url": "mxc://notareal.hs/abcdef",
        }
    });

    let content = from_json_value::<FileEventContent>(json_data).unwrap();
    assert_eq!(content.text.find_plain(), Some("Upload: my_file.txt"));
    assert_eq!(content.text.find_html(), None);
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
}

#[test]
fn encrypted_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": [
            { "body": "Upload: my_file.txt" },
        ],
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
        }
    });

    let content = from_json_value::<FileEventContent>(json_data).unwrap();
    assert_eq!(content.text.find_plain(), Some("Upload: my_file.txt"));
    assert_eq!(content.text.find_html(), None);
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
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
            "m.file": {
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
        "type": "m.file",
    });

    let message_event = assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::File(MessageLikeEvent::Original(message_event))) => message_event
    );
    assert_eq!(message_event.event_id, "$event:notareal.hs");
    let content = message_event.content;
    assert_eq!(content.text.find_plain(), Some("Upload: my_file.txt"));
    assert_eq!(content.text.find_html(), Some("Upload: <strong>my_file.txt</strong>"));
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
    let info = content.file.info.unwrap();
    assert_eq!(info.name.as_deref(), Some("my_file.txt"));
    assert_eq!(info.mimetype.as_deref(), Some("text/plain"));
    assert_eq!(info.size, Some(uint!(774)));
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(134_829_848)));
    assert_eq!(message_event.room_id, "!roomid:notareal.hs");
    assert_eq!(message_event.sender, "@user:notareal.hs");
    assert!(message_event.unsigned.is_empty());
}
