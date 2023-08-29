#![cfg(feature = "unstable-msc3927")]

use std::time::Duration;

use assert_matches2::assert_matches;
use js_int::uint;
use ruma_common::{
    mxc_uri, owned_event_id,
    serde::{Base64, CanBeEmpty},
    MilliSecondsSinceUnixEpoch,
};
#[cfg(feature = "unstable-msc3246")]
use ruma_events::audio::Amplitude;
use ruma_events::{
    audio::{AudioDetailsContentBlock, AudioEventContent},
    file::{EncryptedContentInit, FileContentBlock},
    message::TextContentBlock,
    relation::InReplyTo,
    room::{message::Relation, JsonWebKeyInit},
    AnyMessageLikeEvent, MessageLikeEvent,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[cfg(feature = "unstable-msc3246")]
#[test]
fn amplitude_deserialization_clamp() {
    let json_data = json!(2000);

    let amplitude = from_json_value::<Amplitude>(json_data).unwrap();
    assert_eq!(amplitude.get(), Amplitude::MAX.into());
}

#[test]
fn plain_content_serialization() {
    let event_content = AudioEventContent::with_plain_text(
        "Upload: my_sound.ogg",
        FileContentBlock::plain(
            mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
            "my_sound.ogg".to_owned(),
        ),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "body": "Upload: my_sound.ogg" },
            ],
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "my_sound.ogg",
            },
        })
    );
}

#[test]
fn encrypted_content_serialization() {
    let event_content = AudioEventContent::with_plain_text(
        "Upload: my_sound.ogg",
        FileContentBlock::encrypted(
            mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
            "my_sound.ogg".to_owned(),
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
                { "body": "Upload: my_sound.ogg" },
            ],
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "my_sound.ogg",
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
            },
        })
    );
}

#[test]
fn event_serialization() {
    let mut content = AudioEventContent::new(
        TextContentBlock::html("Upload: my_mix.mp3", "Upload: <strong>my_mix.mp3</strong>"),
        FileContentBlock::plain(
            mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
            "my_mix.mp3".to_owned(),
        ),
    );
    content.file.mimetype = Some("audio/mp3".to_owned());
    content.file.size = Some(uint!(897_774));
    content.audio_details = Some(AudioDetailsContentBlock::new(Duration::from_secs(123)));
    content.relates_to = Some(Relation::Reply {
        in_reply_to: InReplyTo::new(owned_event_id!("$replyevent:example.com")),
    });

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "mimetype": "text/html", "body": "Upload: <strong>my_mix.mp3</strong>" },
                { "body": "Upload: my_mix.mp3"},
            ],
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "my_mix.mp3",
                "mimetype": "audio/mp3",
                "size": 897_774,
            },
            "org.matrix.msc1767.audio_details": {
                "duration": 123,
            },
            "m.relates_to": {
                "m.in_reply_to": {
                    "event_id": "$replyevent:example.com"
                }
            },
        })
    );
}

#[cfg(feature = "unstable-msc3246")]
#[test]
fn plain_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": [
            { "body": "Upload: my_new_song.webm" },
        ],
        "org.matrix.msc1767.file": {
            "url": "mxc://notareal.hs/abcdef",
            "name": "my_new_song.webm",
        },
        "org.matrix.msc1767.audio_details": {
            "duration": 14,
            "org.matrix.msc3246.waveform": [
                13,
                34,
                253,
                234,
                157,
                255,
                1,
                201,
                135,
                125,
                250,
                233,
                231,
                13,
                34,
                252,
                255,
                140,
                187,
                0,
                143,
                235,
                125,
                247,
                183,
                134,
                13,
                34,
                187,
                237,
                145,
                48,
                1,
                66,
                235,
                125,
                204,
                183,
                34,
                13,
                34,
                187,
                237,
                45,
                48,
                1,
                166,
                235,
                125,
                104,
                183,
                234,
            ],
        },
    });

    let content = from_json_value::<AudioEventContent>(json_data).unwrap();
    assert_eq!(content.text.find_plain(), Some("Upload: my_new_song.webm"));
    assert_eq!(content.text.find_html(), None);
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
    assert_eq!(content.file.name, "my_new_song.webm");
    let audio_details = content.audio_details.unwrap();
    assert_eq!(audio_details.duration, Duration::from_secs(14));
    assert_eq!(audio_details.waveform.len(), 52);
}

#[test]
fn encrypted_content_deserialization() {
    let json_data = json!({
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
        },
    });

    let content = from_json_value::<AudioEventContent>(json_data).unwrap();
    assert_eq!(content.text.find_plain(), Some("Upload: my_file.txt"));
    assert_eq!(content.text.find_html(), None);
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
    assert_eq!(content.file.name, "my_file.txt");
    assert!(content.file.encryption_info.is_some());
    assert!(content.audio_details.is_none());
}

#[test]
fn message_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc1767.text": [
                { "body": "Upload: airplane_sound.opus" },
            ],
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "airplane_sound.opus",
                "mimetype": "audio/opus",
                "size": 123_774,
            },
            "org.matrix.msc1767.audio_details": {
                "duration": 53,
            },
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "org.matrix.msc1767.audio",
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::Audio(MessageLikeEvent::Original(message_event))
    );

    assert_eq!(message_event.event_id, "$event:notareal.hs");
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(134_829_848)));
    assert_eq!(message_event.room_id, "!roomid:notareal.hs");
    assert_eq!(message_event.sender, "@user:notareal.hs");
    assert!(message_event.unsigned.is_empty());

    let content = message_event.content;
    assert_eq!(content.text.find_plain(), Some("Upload: airplane_sound.opus"));
    assert_eq!(content.text.find_html(), None);
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
    assert_eq!(content.file.name, "airplane_sound.opus");
    assert_eq!(content.file.mimetype.as_deref(), Some("audio/opus"));
    assert_eq!(content.file.size, Some(uint!(123_774)));
    let audio_details = content.audio_details.unwrap();
    assert_eq!(audio_details.duration, Duration::from_secs(53));
    #[cfg(feature = "unstable-msc3246")]
    assert!(audio_details.waveform.is_empty());
}
