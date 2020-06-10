use std::{
    convert::TryFrom,
    time::{Duration, UNIX_EPOCH},
};

use js_int::UInt;
use matches::assert_matches;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

use ruma_events::{
    call::{answer::AnswerEventContent, SessionDescription, SessionDescriptionType},
    room::{ImageInfo, ThumbnailInfo},
    sticker::StickerEventContent,
    AnyMessageEvent, MessageEvent, UnsignedData,
};

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/07-enum-sanity-check.rs");
    t.compile_fail("tests/ui/08-enum-invalid-path.rs");
}

#[test]
fn deserialize_message_event() {
    let json_data = json!({
        "content": {
            "answer": {
                "type": "answer",
                "sdp": "Hello"
            },
            "call_id": "foofoo",
            "version": 1
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "sender": "@carl:example.com",
        "type": "m.call.answer"
    });

    assert_matches!(
        from_json_value::<AnyMessageEvent>(json_data)
            .unwrap(),
        AnyMessageEvent::CallAnswer(MessageEvent {
            content: AnswerEventContent {
                answer: SessionDescription {
                    session_type: SessionDescriptionType::Answer,
                    sdp,
                },
                call_id,
                version,
            },
            event_id,
            origin_server_ts,
            room_id,
            sender,
            unsigned,
        }) if sdp == "Hello" && call_id == "foofoo" && version == UInt::new(1).unwrap()
            && event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
            && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
            && room_id == RoomId::try_from("!roomid:room.com").unwrap()
            && sender == UserId::try_from("@carl:example.com").unwrap()
            && unsigned.is_empty()
    );
}

#[test]
fn serialize_message_event() {
    let aliases_event = AnyMessageEvent::Sticker(MessageEvent {
        content: StickerEventContent {
            body: "Hello".into(),
            info: ImageInfo {
                height: UInt::new(423),
                width: UInt::new(1011),
                mimetype: Some("image/png".into()),
                size: UInt::new(84242),
                thumbnail_info: Some(Box::new(ThumbnailInfo {
                    width: UInt::new(800),
                    height: UInt::new(334),
                    mimetype: Some("image/png".into()),
                    size: UInt::new(82595),
                })),
                thumbnail_url: Some("mxc://matrix.org".into()),
                thumbnail_file: None,
            },
            url: "http://www.matrix.org".into(),
        },
        event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        room_id: RoomId::try_from("!roomid:room.com").unwrap(),
        sender: UserId::try_from("@carl:example.com").unwrap(),
        unsigned: UnsignedData::default(),
    });

    let actual = to_json_value(&aliases_event).unwrap();
    let expected = json!({
        "content": {
            "body": "Hello",
            "info": {
                "h": 423,
                "mimetype": "image/png",
                "size": 84242,
                "thumbnail_info": {
                "h": 334,
                "mimetype": "image/png",
                "size": 82595,
                "w": 800
                },
                "thumbnail_url": "mxc://matrix.org",
                "w": 1011
            },
            "url": "http://www.matrix.org"
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "sender": "@carl:example.com",
        "type": "m.sticker",
    });

    assert_eq!(actual, expected);
}
