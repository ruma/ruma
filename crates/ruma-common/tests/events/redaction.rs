use assert_matches::assert_matches;
use js_int::uint;
use ruma_common::{
    event_id,
    events::{
        room::redaction::{
            OriginalRoomRedactionEvent, RoomRedactionEvent, RoomRedactionEventContent,
        },
        AnyMessageLikeEvent, MessageLikeUnsigned,
    },
    room_id, user_id, MilliSecondsSinceUnixEpoch,
};
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
    let aliases_event = OriginalRoomRedactionEvent {
        content: RoomRedactionEventContent::with_reason("being a turd".into()),
        redacts: event_id!("$nomore:example.com").to_owned(),
        event_id: event_id!("$h29iv0s8:example.com").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
        room_id: room_id!("!roomid:room.com").to_owned(),
        sender: user_id!("@carl:example.com").to_owned(),
        unsigned: MessageLikeUnsigned::default(),
    };

    let actual = to_json_value(&aliases_event).unwrap();
    let expected = redaction();

    assert_eq!(actual, expected);
}

#[test]
fn deserialize_redaction() {
    let json_data = redaction();

    let ev = assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::RoomRedaction(RoomRedactionEvent::Original(ev))) => ev
    );
    assert_eq!(ev.content.reason.as_deref(), Some("being a turd"));
    assert_eq!(ev.event_id, "$h29iv0s8:example.com");
    assert_eq!(ev.redacts, "$nomore:example.com");
    assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(ev.room_id, "!roomid:room.com");
    assert_eq!(ev.sender, "@carl:example.com");
    assert!(ev.unsigned.is_empty());
}
