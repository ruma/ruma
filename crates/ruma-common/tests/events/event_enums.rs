use js_int::{uint, UInt};
use matches::assert_matches;
use ruma_common::{event_id, room_id, user_id, MilliSecondsSinceUnixEpoch};
use serde_json::{from_value as from_json_value, json};

use ruma_common::events::{
    call::{answer::CallAnswerEventContent, SessionDescription, SessionDescriptionType},
    AnyMessageLikeEvent, MessageLikeEvent,
};

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/events/ui/07-enum-sanity-check.rs");
    t.compile_fail("tests/events/ui/08-enum-invalid-path.rs");
    t.compile_fail("tests/events/ui/09-enum-invalid-kind.rs");
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
        from_json_value::<AnyMessageLikeEvent>(json_data)
            .unwrap(),
        AnyMessageLikeEvent::CallAnswer(MessageLikeEvent {
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
