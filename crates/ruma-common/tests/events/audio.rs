#![cfg(feature = "unstable-msc3246")]

use std::time::Duration;

use assert_matches::assert_matches;
use assign::assign;
use js_int::uint;
use ruma_common::{
    event_id,
    events::{
        audio::{Amplitude, AudioContent, AudioEventContent, Waveform, WaveformError},
        file::{EncryptedContentInit, FileContentBlock},
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
fn waveform_deserialization_pass() {
    let json_data = json!([
        13, 34, 987, 937, 345, 648, 1, 366, 235, 125, 904, 783, 734, 13, 34, 987, 937, 345, 648, 1,
        366, 235, 125, 904, 783, 734, 13, 34, 987, 937, 345, 648, 1, 366, 235, 125, 904, 783, 734,
        13, 34, 987, 937, 345, 648, 1, 366, 235, 125, 904, 783, 734,
    ]);

    let waveform = from_json_value::<Waveform>(json_data).unwrap();
    assert_eq!(waveform.amplitudes().len(), 52);
}

#[test]
fn waveform_deserialization_not_enough() {
    let json_data = json!([]);

    let err = from_json_value::<Waveform>(json_data).unwrap_err();
    assert!(err.is_data());
    assert_eq!(err.to_string(), WaveformError::NotEnoughValues.to_string());
}

#[test]
fn waveform_deserialization_clamp_amplitude() {
    let json_data = json!([
        2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000,
        2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000,
    ]);

    let waveform = from_json_value::<Waveform>(json_data).unwrap();
    assert!(waveform.amplitudes().iter().all(|amp| amp.get() == Amplitude::MAX.into()));
}

#[test]
fn plain_content_serialization() {
    let event_content = AudioEventContent::plain(
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
            "m.audio": {}
        })
    );
}

#[test]
fn encrypted_content_serialization() {
    let event_content = AudioEventContent::plain(
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
            "m.audio": {}
        })
    );
}

#[test]
fn event_serialization() {
    let content = assign!(
        AudioEventContent::new(
            TextContentBlock::html(
                "Upload: my_mix.mp3",
                "Upload: <strong>my_mix.mp3</strong>",
            ),
            assign!(
                FileContentBlock::plain(
                    mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
                    "my_mix.mp3".to_owned()
                ),
                {
                    mimetype: Some("audio/mp3".to_owned()),
                    size: Some(uint!(897_774)),
                }
            )
        ),
        {
            audio: assign!(
                AudioContent::new(),
                {
                    duration: Some(Duration::from_secs(123))
                }
            ),
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo::new(event_id!("$replyevent:example.com").to_owned()),
            }),
        }
    );

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
            "m.audio": {
                "duration": 123_000,
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
            { "body": "Upload: my_new_song.webm" },
        ],
        "org.matrix.msc1767.file": {
            "url": "mxc://notareal.hs/abcdef",
            "name": "my_new_song.webm",
        },
        "m.audio": {
            "waveform": [
                13,
                34,
                987,
                937,
                345,
                648,
                1,
                366,
                235,
                125,
                904,
                783,
                734,
                13,
                34,
                987,
                937,
                345,
                648,
                1,
                366,
                235,
                125,
                904,
                783,
                734,
                13,
                34,
                987,
                937,
                345,
                648,
                1,
                366,
                235,
                125,
                904,
                783,
                734,
                13,
                34,
                987,
                937,
                345,
                648,
                1,
                366,
                235,
                125,
                904,
                783,
                734,
            ],
        }
    });

    let content = from_json_value::<AudioEventContent>(json_data).unwrap();
    assert_eq!(content.text.find_plain(), Some("Upload: my_new_song.webm"));
    assert_eq!(content.text.find_html(), None);
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
    assert_eq!(content.file.name, "my_new_song.webm");
    let waveform = content.audio.waveform.unwrap();
    assert_eq!(waveform.amplitudes().len(), 52);
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
            "v": "v2"
        },
        "m.audio": {},
    });

    let content = from_json_value::<AudioEventContent>(json_data).unwrap();
    assert_eq!(content.text.find_plain(), Some("Upload: my_file.txt"));
    assert_eq!(content.text.find_html(), None);
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
    assert_eq!(content.file.name, "my_file.txt");
    assert!(content.file.encryption_info.is_some());
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
            "m.audio": {
                "duration": 5_300,
            }
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.audio",
    });

    let message_event = assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::Audio(MessageLikeEvent::Original(message_event)) => message_event
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
    assert_eq!(content.audio.duration, Some(Duration::from_millis(5_300)));
    assert_matches!(content.audio.waveform, None);
}
