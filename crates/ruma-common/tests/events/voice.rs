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
        relation::InReplyTo,
        room::message::Relation,
        voice::VoiceEventContent,
        AnyMessageLikeEvent, MessageLikeEvent,
    },
    mxc_uri,
    serde::CanBeEmpty,
    MilliSecondsSinceUnixEpoch,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn event_serialization() {
    let content = assign!(
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
    );

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "body": "Voice message" },
            ],
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
            },
        })
    );
}

#[test]
fn message_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc1767.text": [
                { "body": "Voice message" },
            ],
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

    let ev = assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::Voice(MessageLikeEvent::Original(ev))) => ev
    );
    assert_eq!(ev.event_id, "$event:notareal.hs");
    assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(134_829_848)));
    assert_eq!(ev.room_id, "!roomid:notareal.hs");
    assert_eq!(ev.sender, "@user:notareal.hs");
    assert!(ev.unsigned.is_empty());

    let content = ev.content;
    assert_eq!(content.text.find_plain(), Some("Voice message"));
    assert_eq!(content.text.find_html(), None);
    assert_eq!(content.file.url, "mxc://notareal.hs/abcdef");
    assert_eq!(content.audio.duration, Some(Duration::from_millis(5_300)));
    assert_matches!(content.audio.waveform, None);

    let info = content.file.info.unwrap();
    assert_eq!(info.name.as_deref(), Some("voice_message.ogg"));
    assert_eq!(info.mimetype.as_deref(), Some("audio/opus"));
    assert_eq!(info.size, Some(uint!(123_774)));
}
