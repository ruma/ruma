#![cfg(feature = "unstable-msc3245")]

use std::time::Duration;

use assign::assign;
use js_int::uint;
use matches::assert_matches;
use ruma_common::{
    event_id,
    events::{
        audio::AudioContent,
        file::{FileContent, FileContentInfo},
        room::message::{InReplyTo, Relation},
        voice::VoiceEventContent,
        AnyMessageLikeEvent, MessageLikeEvent, MessageLikeUnsigned,
    },
    mxc_uri, room_id, user_id, MilliSecondsSinceUnixEpoch,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn event_serialization() {
    let event = MessageLikeEvent {
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
                "org.matrix.msc1767.file": {
                    "url": "mxc://notareal.hs/abcdef",
                    "name": "voice_message.ogg",
                    "mimetype": "audio/opus",
                    "size": 897_774,
                },
                "org.matrix.msc1767.audio": {
                    "duration": 23_000,
                },
                "org.matrix.msc3245.voice": {},
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
            "org.matrix.msc1767.text": "Voice message",
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "voice_message.ogg",
                "mimetype": "audio/opus",
                "size": 123_774,
            },
            "org.matrix.msc1767.audio": {
                "duration": 5_300,
            },
            "org.matrix.msc3245.voice": {},
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.voice",
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::Voice(MessageLikeEvent {
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
        }) if event_id == event_id!("$event:notareal.hs")
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
