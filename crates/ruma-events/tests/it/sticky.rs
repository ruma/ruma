#[cfg(feature = "unstable-msc4354")]
use assert_matches2::assert_matches;
use js_int::uint;
use ruma_common::{serde::CanBeEmpty, MilliSecondsSinceUnixEpoch};
use ruma_events::{sticky::StickyDurationMs, AnyMessageLikeEvent, MessageLikeEvent};
use serde_json::{from_value as from_json_value, json};

#[test]
fn deserialize_sticky_event() {
    let json_data = json!({
        "content": {
            "body": "Hello, but sticky",
            "msgtype": "m.text",
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "sender": "@alice:example.com",
        "type": "m.room.message",
        "msc4354_sticky": {
            "duration_ms": 3600000
        }
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::RoomMessage(MessageLikeEvent::Original(message_event))
    );

    assert_eq!(message_event.event_id, "$h29iv0s8:example.com");
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(message_event.room_id, "!roomid:room.com");
    assert_eq!(message_event.sender, "@alice:example.com");
    assert!(message_event.unsigned.is_empty());

    let content = message_event.content;
    assert_eq!(content.body(), "Hello, but sticky");

    assert!(message_event.msc4354_sticky.is_some());
}

#[test]
fn deserialize_sticky_event_to_high() {
    let json_data = json!({
        "content": {
            "body": "Hello, but sticky",
            "msgtype": "m.text",
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "sender": "@alice:example.com",
        "type": "m.room.message",
        "msc4354_sticky": {
            "duration_ms": 4600000
        }
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::RoomMessage(MessageLikeEvent::Original(message_event))
    );

    let content = message_event.content;
    assert_eq!(content.body(), "Hello, but sticky");

    assert!(message_event.msc4354_sticky.is_some());
    assert_eq!(
        message_event.msc4354_sticky.unwrap().clamped_duration_ms(),
        StickyDurationMs::new(3600000).expect("valid duration")
    );
}
#[test]
fn deserialize_sticky_event_default() {
    let json_data = json!({
        "content": {
            "body": "Hello, but sticky",
            "msgtype": "m.text",
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "sender": "@alice:example.com",
        "type": "m.room.message",
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::RoomMessage(MessageLikeEvent::Original(message_event))
    );

    assert!(message_event.msc4354_sticky.is_none());
}
