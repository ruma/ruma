#![cfg(feature = "unstable-msc3245")]

use std::time::Duration;

use assert_matches::assert_matches;
use assign::assign;
use js_int::uint;
use ruma_common::{
    event_id,
    events::{
        audio::AudioContent,
        file::{FileContent, FileContentInfo},
        room::{
            message::{
                AudioMessageEventContent, InReplyTo, MessageType, Relation, RoomMessageEventContent,
            },
            MediaSource,
        },
        voice::{VoiceContent, VoiceEventContent},
        AnyMessageLikeEvent, MessageLikeEvent, MessageLikeUnsigned, OriginalMessageLikeEvent,
    },
    mxc_uri, room_id, user_id, MilliSecondsSinceUnixEpoch,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn event_serialization() {
    let event = OriginalMessageLikeEvent {
        content: assign!(
            VoiceEventContent::plain(
                "Voice message",
                FileContent::plain(
                    mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
                    Some(Box::new(assign!(
                        FileContentInfo::new(),
                        {
                            name: Some("voice_message.ogg".to_owned()),
                            mimetype: Some("audio/opus".to_owned()),
                            size: Some(uint!(897_774)),
                        }
                    ))),
                )
            ),
            {
                audio: assign!(
                    AudioContent::new(),
                    {
                        duration: Some(Duration::from_secs(23))
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
                "org.matrix.msc1767.text": "Voice message",
                "m.file": {
                    "url": "mxc://notareal.hs/abcdef",
                    "name": "voice_message.ogg",
                    "mimetype": "audio/opus",
                    "size": 897_774,
                },
                "m.audio": {
                    "duration": 23_000,
                },
                "m.voice": {},
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
            "type": "m.voice",
        })
    );
}

#[test]
fn message_event_deserialization() {
    let json_data = json!({
        "content": {
            "m.text": "Voice message",
            "m.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "voice_message.ogg",
                "mimetype": "audio/opus",
                "size": 123_774,
            },
            "m.audio": {
                "duration": 5_300,
            },
            "m.voice": {},
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.voice",
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::Voice(MessageLikeEvent::Original(OriginalMessageLikeEvent {
            content: VoiceEventContent {
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
        })) if event_id == event_id!("$event:notareal.hs")
            && message.find_plain() == Some("Voice message")
            && message.find_html().is_none()
            && url == "mxc://notareal.hs/abcdef"
            && info.name.as_deref() == Some("voice_message.ogg")
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

#[test]
fn room_message_serialization() {
    let message_event_content = RoomMessageEventContent::new(MessageType::Audio(assign!(
        AudioMessageEventContent::plain(
            "Upload: voice_message.ogg".to_owned(),
            mxc_uri!("mxc://notareal.hs/file").to_owned(),
            None,
        ), {
            voice: Some(VoiceContent::new()),
        }
    )));

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "Upload: voice_message.ogg",
            "url": "mxc://notareal.hs/file",
            "msgtype": "m.audio",
            "org.matrix.msc1767.text": "Upload: voice_message.ogg",
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/file",
            },
            "org.matrix.msc1767.audio": {},
            "org.matrix.msc3245.voice": {},
        })
    );
}

#[test]
fn room_message_stable_deserialization() {
    let json_data = json!({
        "body": "Upload: voice_message.ogg",
        "url": "mxc://notareal.hs/file",
        "msgtype": "m.audio",
        "m.text": "Upload: voice_message.ogg",
        "m.file": {
            "url": "mxc://notareal.hs/file",
        },
        "m.audio": {},
        "m.voice": {},
    });

    let event_content = from_json_value::<RoomMessageEventContent>(json_data).unwrap();
    assert_matches!(event_content.msgtype, MessageType::Audio(_));
    if let MessageType::Audio(content) = event_content.msgtype {
        assert_eq!(content.body, "Upload: voice_message.ogg");
        assert_matches!(content.source, MediaSource::Plain(_));
        if let MediaSource::Plain(url) = content.source {
            assert_eq!(url, "mxc://notareal.hs/file");
        }
        let message = content.message.unwrap();
        assert_eq!(message.len(), 1);
        assert_eq!(message[0].body, "Upload: voice_message.ogg");
        let file = content.file.unwrap();
        assert_eq!(file.url, "mxc://notareal.hs/file");
        assert!(!file.is_encrypted());
        assert!(content.voice.is_some());
    }
}

#[test]
fn room_message_unstable_deserialization() {
    let json_data = json!({
        "body": "Upload: voice_message.ogg",
        "url": "mxc://notareal.hs/file",
        "msgtype": "m.audio",
        "org.matrix.msc1767.text": "Upload: voice_message.ogg",
        "org.matrix.msc1767.file": {
            "url": "mxc://notareal.hs/file",
        },
        "org.matrix.msc1767.audio": {},
        "org.matrix.msc3245.voice": {},
    });

    let event_content = from_json_value::<RoomMessageEventContent>(json_data).unwrap();
    assert_matches!(event_content.msgtype, MessageType::Audio(_));
    if let MessageType::Audio(content) = event_content.msgtype {
        assert_eq!(content.body, "Upload: voice_message.ogg");
        assert_matches!(content.source, MediaSource::Plain(_));
        if let MediaSource::Plain(url) = content.source {
            assert_eq!(url, "mxc://notareal.hs/file");
        }
        let message = content.message.unwrap();
        assert_eq!(message.len(), 1);
        assert_eq!(message[0].body, "Upload: voice_message.ogg");
        let file = content.file.unwrap();
        assert_eq!(file.url, "mxc://notareal.hs/file");
        assert!(!file.is_encrypted());
        assert!(content.voice.is_some());
    }
}
