use assert_matches2::assert_matches;
use js_int::uint;
use ruma_common::{owned_event_id, serde::CanBeEmpty, MilliSecondsSinceUnixEpoch, RoomVersionId};
use ruma_events::{
    room::redaction::{RoomRedactionEvent, RoomRedactionEventContent},
    AnyMessageLikeEvent,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn serialize_redaction_content() {
    let content = RoomRedactionEventContent::new_v1().with_reason("being very unfriendly".into());

    let actual = to_json_value(content).unwrap();
    let expected = json!({
        "reason": "being very unfriendly"
    });

    assert_eq!(actual, expected);
}

#[test]
fn serialize_redaction_content_v11() {
    let redacts = owned_event_id!("$abcdef");
    let content = RoomRedactionEventContent::new_v11(redacts.clone())
        .with_reason("being very unfriendly".into());

    let actual = to_json_value(content).unwrap();
    let expected = json!({
        "redacts": redacts,
        "reason": "being very unfriendly"
    });

    assert_eq!(actual, expected);
}

#[test]
fn deserialize_redaction() {
    let json_data = json!({
        "content": {
            "redacts": "$nomorev11:example.com",
            "reason": "being very unfriendly"
        },
        "redacts": "$nomorev1:example.com",
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

    assert_eq!(ev.redacts(&RoomVersionId::V1), "$nomorev1:example.com");
    assert_eq!(ev.redacts(&RoomVersionId::V11), "$nomorev11:example.com");

    assert_eq!(ev.content.redacts.unwrap(), "$nomorev11:example.com");
    assert_eq!(ev.content.reason.as_deref(), Some("being very unfriendly"));
    assert_eq!(ev.event_id, "$h29iv0s8:example.com");
    assert_eq!(ev.redacts.unwrap(), "$nomorev1:example.com");
    assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(ev.room_id, "!roomid:room.com");
    assert_eq!(ev.sender, "@carl:example.com");
    assert!(ev.unsigned.is_empty());
}

#[test]
fn deserialize_redaction_missing_redacts() {
    let json_data = json!({
        "content": {
            "reason": "being very unfriendly"
        },
        "event_id": "$h29iv0s8:example.com",
        "sender": "@carl:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "type": "m.room.redaction"
    });

    from_json_value::<AnyMessageLikeEvent>(json_data).unwrap_err();
}
