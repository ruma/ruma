#![cfg(feature = "unstable-msc3245")]

use std::time::Duration;

use assert_matches::assert_matches;
use js_int::uint;
use ruma_common::{
    event_id,
    events::{
        audio::AudioDetailsContentBlock, file::FileContentBlock, relation::InReplyTo,
        room::message::Relation, voice::VoiceEventContent, AnyMessageLikeEvent, MessageLikeEvent,
    },
    mxc_uri,
    serde::CanBeEmpty,
    MilliSecondsSinceUnixEpoch,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn event_serialization() {
    let mut content = VoiceEventContent::plain(
        "Voice message",
        FileContentBlock::plain(
            mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
            "voice_message.ogg".to_owned(),
        ),
    );

    content.file.mimetype = Some("audio/opus".to_owned());
    content.file.size = Some(uint!(897_774));
    content.audio_details = Some(AudioDetailsContentBlock::new(Duration::from_secs(23)));
    content.relates_to = Some(Relation::Reply {
        in_reply_to: InReplyTo::new(event_id!("$replyevent:example.com").to_owned()),
    });

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "body": "Voice message" },
            ],
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "voice_message.ogg",
                "mimetype": "audio/opus",
                "size": 897_774,
            },
            "org.matrix.msc1767.audio_details": {
                "duration": 23,
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
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/abcdef",
                "name": "voice_message.ogg",
                "mimetype": "audio/opus",
                "size": 123_774,
            },
            "org.matrix.msc1767.audio_details": {
                "duration": 53,
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
    assert_eq!(content.file.name, "voice_message.ogg");
    assert_eq!(content.file.mimetype.as_deref(), Some("audio/opus"));
    assert_eq!(content.file.size, Some(uint!(123_774)));
    let audio_details = content.audio_details.unwrap();
    assert_eq!(audio_details.duration, Duration::from_secs(53));
    assert!(audio_details.waveform.is_empty());
}
