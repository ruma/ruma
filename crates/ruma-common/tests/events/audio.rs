#![cfg(feature = "unstable-msc3246")]

use std::time::Duration;

use assign::assign;
use js_int::uint;
use matches::assert_matches;
use ruma_common::{
    event_id,
    events::{
        audio::{Amplitude, AudioContent, AudioEventContent, Waveform, WaveformError},
        file::{EncryptedContentInit, FileContent, FileContentInfo},
        message::MessageContent,
        room::{
            message::{InReplyTo, Relation},
            JsonWebKeyInit,
        },
        AnyMessageLikeEvent, MessageLikeEvent, MessageLikeUnsigned,
    },
    mxc_uri, room_id,
    serde::Base64,
    user_id, MilliSecondsSinceUnixEpoch,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn waveform_deserialization_pass() {
    let json_data = json!([
        13, 34, 987, 937, 345, 648, 1, 366, 235, 125, 904, 783, 734, 13, 34, 987, 937, 345, 648, 1,
        366, 235, 125, 904, 783, 734, 13, 34, 987, 937, 345, 648, 1, 366, 235, 125, 904, 783, 734,
        13, 34, 987, 937, 345, 648, 1, 366, 235, 125, 904, 783, 734,
    ]);

    assert_matches!(
        from_json_value::<Waveform>(json_data),
        Ok(waveform) if waveform.amplitudes().len() == 52
    );
}

#[test]
fn waveform_deserialization_not_enough() {
    let json_data = json!([]);

    assert_matches!(
        from_json_value::<Waveform>(json_data),
        Err(err)
            if err.is_data()
            && format!("{}", err) == format!("{}", WaveformError::NotEnoughValues)
    );
}

#[test]
fn waveform_deserialization_clamp_amplitude() {
    let json_data = json!([
        2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000,
        2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000, 2000,
    ]);

    assert_matches!(
        from_json_value::<Waveform>(json_data).unwrap(),
        waveform if waveform.amplitudes().iter().all(|amp| amp.value() == Amplitude::MAX.into())
    );
}

#[test]
fn plain_content_serialization() {
    let event_content = AudioEventContent::plain(
        "Upload: my_sound.ogg",
        FileContent::plain(mxc_uri!("mxc://notareal.hs/abcdef").to_owned(), None),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": "Upload: my_sound.ogg",
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
            },
            "org.matrix.msc1767.audio": {}
        })
    );
}

#[test]
fn encrypted_content_serialization() {
    let event_content = AudioEventContent::plain(
        "Upload: my_sound.ogg",
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
            "org.matrix.msc1767.text": "Upload: my_sound.ogg",
            "org.matrix.msc1767.file": {
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
            "org.matrix.msc1767.audio": {}
        })
    );
}

#[test]
fn event_serialization() {
    let event = MessageLikeEvent {
        content: assign!(
            AudioEventContent::with_message(
                MessageContent::html(
                    "Upload: my_mix.mp3",
                    "Upload: <strong>my_mix.mp3</strong>",
                ),
                FileContent::plain(
                    mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
                    Some(Box::new(assign!(
                        FileContentInfo::new(),
                        {
                            name: Some("my_mix.mp3".to_owned()),
                            mimetype: Some("audio/mp3".to_owned()),
                            size: Some(uint!(897_774)),
                        }
                    ))),
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
                    { "body": "Upload: <strong>my_mix.mp3</strong>", "mimetype": "text/html"},
                    { "body": "Upload: my_mix.mp3", "mimetype": "text/plain"},
                ],
                "org.matrix.msc1767.file": {
                    "url": "mxc://notareal.hs/abcdef",
                    "name": "my_mix.mp3",
                    "mimetype": "audio/mp3",
                    "size": 897_774,
                },
                "org.matrix.msc1767.audio": {
                    "duration": 123_000,
                },
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
            "type": "m.audio",
        })
    );
}

#[test]
fn plain_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": "Upload: my_new_song.webm",
        "org.matrix.msc1767.file": {
            "url": "mxc://notareal.hs/abcdef",
        },
        "org.matrix.msc1767.audio": {
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

    assert_matches!(
        from_json_value::<AudioEventContent>(json_data)
            .unwrap(),
        AudioEventContent {
            message,
            file,
            audio: AudioContent { duration: None, waveform: Some(waveform), .. },
            ..
        }
        if message.find_plain() == Some("Upload: my_new_song.webm")
            && message.find_html().is_none()
            && file.url == "mxc://notareal.hs/abcdef"
            && waveform.amplitudes().len() == 52
    );
}

#[test]
fn encrypted_content_deserialization() {
    let json_data = json!({
        "org.matrix.msc1767.text": "Upload: my_file.txt",
        "org.matrix.msc1767.file": {
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
        "org.matrix.msc1767.audio": {},
    });

    assert_matches!(
        from_json_value::<AudioEventContent>(json_data)
            .unwrap(),
        AudioEventContent {
            message,
            file,
            audio: AudioContent { duration: None, waveform: None, .. },
            ..
        }
        if message.find_plain() == Some("Upload: my_file.txt")
            && message.find_html().is_none()
            && file.url == "mxc://notareal.hs/abcdef"
            && file.encryption_info.is_some()
    );
}

#[test]
fn message_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc1767.text": "Upload: airplane_sound.opus",
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "airplane_sound.opus",
                "mimetype": "audio/opus",
                "size": 123_774,
            },
            "org.matrix.msc1767.audio": {
                "duration": 5_300,
            }
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.audio",
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::Audio(MessageLikeEvent {
            content: AudioEventContent {
                message,
                file: FileContent {
                    url,
                    info: Some(info),
                    ..
                },
                audio,
                ..
            },
            event_id,
            origin_server_ts,
            room_id,
            sender,
            unsigned
        }) if event_id == event_id!("$event:notareal.hs")
            && message.find_plain() == Some("Upload: airplane_sound.opus")
            && message.find_html().is_none()
            && url == "mxc://notareal.hs/abcdef"
            && info.name.as_deref() == Some("airplane_sound.opus")
            && info.mimetype.as_deref() == Some("audio/opus")
            && info.size == Some(uint!(123_774))
            && audio.duration == Some(Duration::from_millis(5_300))
            && audio.waveform.is_none()
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(134_829_848))
            && room_id == room_id!("!roomid:notareal.hs")
            && sender == user_id!("@user:notareal.hs")
            && unsigned.is_empty()
    );
}
