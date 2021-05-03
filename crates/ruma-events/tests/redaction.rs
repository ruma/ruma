use std::time::{Duration, UNIX_EPOCH};

use matches::assert_matches;
use ruma_events::{
    room::redaction::{RedactionEvent, RedactionEventContent},
    AnyMessageEvent, Unsigned,
};
use ruma_identifiers::{event_id, room_id, user_id};
use ruma_serde::Raw;
use serde_json::{
    from_value as from_json_value, json, to_value as to_json_value, Value as JsonValue,
};

fn redaction() -> JsonValue {
    json!({
        "content": {
            "reason": "being a turd"
        },
        "redacts": "$nomore:example.com",
        "event_id": "$h29iv0s8:example.com",
        "sender": "@carl:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "type": "m.room.redaction"
    })
}

#[test]
fn serialize_redaction() {
    let aliases_event = RedactionEvent {
        content: RedactionEventContent { reason: Some("being a turd".into()) },
        redacts: event_id!("$nomore:example.com"),
        event_id: event_id!("$h29iv0s8:example.com"),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        room_id: room_id!("!roomid:room.com"),
        sender: user_id!("@carl:example.com"),
        unsigned: Unsigned::default(),
    };

    let actual = to_json_value(&aliases_event).unwrap();
    let expected = redaction();

    assert_eq!(actual, expected);
}

#[test]
fn deserialize_redaction() {
    let json_data = redaction();

    assert_matches!(
        from_json_value::<Raw<AnyMessageEvent>>(json_data)
            .unwrap()
            .deserialize()
            .unwrap(),
        AnyMessageEvent::RoomRedaction(RedactionEvent {
            content: RedactionEventContent { reason: Some(reas) },
            redacts,
            event_id,
            origin_server_ts,
            room_id,
            sender,
            unsigned,
        }) if reas == "being a turd"
            && event_id == event_id!("$h29iv0s8:example.com")
            && redacts == event_id!("$nomore:example.com")
            && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
            && room_id == room_id!("!roomid:room.com")
            && sender == user_id!("@carl:example.com")
            && unsigned.is_empty()
    );
}
