use assert_matches2::assert_matches;
use js_int::uint;
use ruma_common::{serde::CanBeEmpty, MilliSecondsSinceUnixEpoch, VoipVersionId};
use ruma_events::{AnyMessageLikeEvent, MessageLikeEvent};
use serde_json::{from_value as from_json_value, json};

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/it/ui/07-enum-sanity-check.rs");
    t.compile_fail("tests/it/ui/08-enum-invalid-path.rs");
    t.compile_fail("tests/it/ui/09-enum-invalid-kind.rs");
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
            "version": 0
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "sender": "@carl:example.com",
        "type": "m.call.answer"
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::CallAnswer(MessageLikeEvent::Original(message_event))
    );

    assert_eq!(message_event.event_id, "$h29iv0s8:example.com");
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(message_event.room_id, "!roomid:room.com");
    assert_eq!(message_event.sender, "@carl:example.com");
    assert!(message_event.unsigned.is_empty());

    let content = message_event.content;
    assert_eq!(content.answer.sdp, "Hello");
    assert_eq!(content.call_id, "foofoo");
    assert_eq!(content.version, VoipVersionId::V0);
}
