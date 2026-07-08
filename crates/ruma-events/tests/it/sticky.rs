use std::time::Duration;

use assert_matches2::assert_matches;
use js_int::uint;
use ruma_common::{MilliSecondsSinceUnixEpoch, serde::CanBeEmpty};
use ruma_events::{AnyMessageLikeEvent, MessageLikeEvent, sticky::StickyDurationMs};
use serde_json::{from_value as from_json_value, json};

#[test]
fn new_clamped_keeps_in_range_values() {
    let d = StickyDurationMs::new_clamped(42_u32);
    assert_eq!(d.get(), 42);
}

#[test]
fn new_clamped_clamps_to_max_for_just_over_max() {
    let d = StickyDurationMs::new_clamped(3_600_000_u32 + 10_000);
    assert_eq!(d.get(), 3_600_000);
}

#[test]
fn new_clamped_clamps_large_values_to_max() {
    let d = StickyDurationMs::new_clamped(u64::MAX);
    assert_eq!(d.get(), 3_600_000);
}

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
            "duration_ms": 3_600_000
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

    assert!(message_event.sticky.is_some());
    assert_eq!(message_event.sticky.map(|s| s.duration_ms.get()), Some(3_600_000));
}

#[test]
fn deserialize_sticky_top_level_support_server_sends_both_stable_unstable() {
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
        "sticky": {
            "duration_ms": 30_000
        },
        "msc4354_sticky": {
            "duration_ms": 30_000
        },
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::RoomMessage(MessageLikeEvent::Original(message_event))
    );

    assert!(message_event.sticky.is_some());
    assert_eq!(message_event.sticky.map(|s| s.duration_ms.get()), Some(30_000));
}

#[test]
fn deserialize_sticky_out_of_range() {
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
            "duration_ms": 4_600_000
        }
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::RoomMessage(MessageLikeEvent::Original(message_event))
    );

    let content = message_event.content;
    assert_eq!(content.body(), "Hello, but sticky");

    assert!(message_event.sticky.is_none());
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

    assert!(message_event.sticky.is_none());
}

#[test]
fn deserialize_sticky_duration_ttl_ms() {
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
            "duration_ms": 3_600_000
        },
        "unsigned": {
            "msc4354_sticky_duration_ttl_ms": 42_000
        }
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::RoomMessage(MessageLikeEvent::Original(message_event))
    );

    assert_eq!(message_event.unsigned.sticky_duration_ttl_ms, Some(Duration::from_millis(42_000)));
    assert!(!message_event.unsigned.is_empty());
}

#[test]
fn deserialize_sticky_duration_ttl_ms_support_server_sends_both_stable_unstable() {
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
        "sticky": {
            "duration_ms": 3_600_000
        },
        "unsigned": {
            "sticky_duration_ttl_ms": 42_000,
            "msc4354_sticky_duration_ttl_ms": 42_000,
        }
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::RoomMessage(MessageLikeEvent::Original(message_event))
    );

    assert_eq!(message_event.unsigned.sticky_duration_ttl_ms, Some(Duration::from_millis(42_000)));
    assert!(!message_event.unsigned.is_empty());
}
