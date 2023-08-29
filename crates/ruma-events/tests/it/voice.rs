#![cfg(feature = "unstable-msc3245")]

use std::time::Duration;

use assert_matches2::assert_matches;
use js_int::uint;
use ruma_common::{mxc_uri, owned_event_id, serde::CanBeEmpty, MilliSecondsSinceUnixEpoch};
use ruma_events::{
    audio::Amplitude,
    file::FileContentBlock,
    relation::InReplyTo,
    room::message::Relation,
    voice::{VoiceAudioDetailsContentBlock, VoiceEventContent},
    AnyMessageLikeEvent, MessageLikeEvent,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn event_serialization() {
    let mut content = VoiceEventContent::with_plain_text(
        "Voice message",
        FileContentBlock::plain(
            mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
            "voice_message.ogg".to_owned(),
        ),
        VoiceAudioDetailsContentBlock::new(
            Duration::from_secs(23),
            vec![Amplitude::from(255), Amplitude::from(0)],
        ),
    );

    content.file.mimetype = Some("audio/opus".to_owned());
    content.file.size = Some(uint!(897_774));
    content.relates_to = Some(Relation::Reply {
        in_reply_to: InReplyTo::new(owned_event_id!("$replyevent:example.com")),
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
                "org.matrix.msc3246.waveform": [255, 0],
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
                "org.matrix.msc3246.waveform": [255, 0],
            },
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "org.matrix.msc3245.voice.v2",
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::Voice(MessageLikeEvent::Original(ev)))
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
    assert_eq!(content.audio_details.duration, Duration::from_secs(53));
    assert_eq!(content.audio_details.waveform.len(), 2);
}
