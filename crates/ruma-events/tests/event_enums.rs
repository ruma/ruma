use js_int::{uint, UInt};
use matches::assert_matches;
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_identifiers::{event_id, room_id, user_id};
use serde_json::{from_value as from_json_value, json};

use ruma_events::{
    call::{answer::CallAnswerEventContent, SessionDescription, SessionDescriptionType},
    AnyMessageEvent, MessageEvent,
};

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/07-enum-sanity-check.rs");
    t.compile_fail("tests/ui/08-enum-invalid-path.rs");
    t.compile_fail("tests/ui/09-enum-invalid-kind.rs");
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
            content: CallAnswerEventContent {
                answer: SessionDescription {
                    session_type: SessionDescriptionType::Answer,
                    sdp,
                    ..
                },
                call_id,
                version,
                ..
            },
            event_id,
            origin_server_ts,
            room_id,
            sender,
            unsigned,
        }) if sdp == "Hello" && call_id == "foofoo" && version == UInt::new(1).unwrap()
            && event_id == event_id!("$h29iv0s8:example.com")
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(1))
            && room_id == room_id!("!roomid:room.com")
            && sender == user_id!("@carl:example.com")
            && unsigned.is_empty()
    );
}
