use assert_matches2::assert_matches;
use js_int::uint;
use ruma_common::{
    events::{
        room::redaction::{RoomRedactionEvent, RoomRedactionEventContent},
        AnyMessageLikeEvent,
    },
    serde::CanBeEmpty,
    MilliSecondsSinceUnixEpoch,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn serialize_redaction_content() {
    let content = RoomRedactionEventContent::with_reason("being very unfriendly".into());

    let actual = to_json_value(content).unwrap();
    let expected = json!({
        "reason": "being very unfriendly"
    });

    assert_eq!(actual, expected);
}

#[test]
fn deserialize_redaction() {
    let json_data = json!({
        "content": {
            "reason": "being very unfriendly"
        },
        "redacts": "$nomore:example.com",
        "event_id": "$h29iv0s8:example.com",
        "sender": "@carl:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "type": "m.room.redaction"
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::RoomRedaction(RoomRedactionEvent::Original(ev)))
    );
    assert_eq!(ev.content.reason.as_deref(), Some("being very unfriendly"));
    assert_eq!(ev.event_id, "$h29iv0s8:example.com");
    assert_eq!(ev.redacts, "$nomore:example.com");
    assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(ev.room_id, "!roomid:room.com");
    assert_eq!(ev.sender, "@carl:example.com");
    assert!(ev.unsigned.is_empty());
}
