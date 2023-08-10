use assert_matches2::assert_matches;
use js_int::uint;
use ruma_common::{
    events::{
        room::redaction::{
            v1::RoomRedactionV1EventContent, v11::RoomRedactionV11EventContent,
            OriginalRoomRedactionEvent, RoomRedactionEvent,
        },
        AnyMessageLikeEvent,
    },
    owned_event_id,
    serde::CanBeEmpty,
    MilliSecondsSinceUnixEpoch, RoomVersionId,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn serialize_redaction_v1_content() {
    let content = RoomRedactionV1EventContent::with_reason("being very unfriendly".into());

    let actual = to_json_value(content).unwrap();
    let expected = json!({
        "reason": "being very unfriendly"
    });

    assert_eq!(actual, expected);
}

#[test]
fn deserialize_redaction_v1() {
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
        Ok(AnyMessageLikeEvent::RoomRedaction(ev))
    );
    assert_eq!(ev.redacts(RoomVersionId::V1).unwrap(), "$nomore:example.com");
    assert_eq!(ev.redacts(RoomVersionId::V11), None);

    assert_matches!(ev, RoomRedactionEvent::Original(OriginalRoomRedactionEvent::V1(v1_ev)));
    assert_eq!(v1_ev.content.reason.as_deref(), Some("being very unfriendly"));
    assert_eq!(v1_ev.redacts, "$nomore:example.com");
    assert_eq!(v1_ev.event_id, "$h29iv0s8:example.com");
    assert_eq!(v1_ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(v1_ev.room_id, "!roomid:room.com");
    assert_eq!(v1_ev.sender, "@carl:example.com");
    assert!(v1_ev.unsigned.is_empty());
}

#[test]
fn serialize_redaction_v11_content() {
    let redacts = owned_event_id!("$abcdef");
    let content =
        RoomRedactionV11EventContent::with_reason(redacts.clone(), "being very unfriendly".into());

    let actual = to_json_value(content).unwrap();
    let expected = json!({
        "redacts": redacts,
        "reason": "being very unfriendly"
    });

    assert_eq!(actual, expected);
}

#[test]
fn deserialize_redaction_v11() {
    let json_data = json!({
        "content": {
            "redacts": "$nomore:example.com",
            "reason": "being very unfriendly",
        },
        "event_id": "$h29iv0s8:example.com",
        "sender": "@carl:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "type": "m.room.redaction",
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::RoomRedaction(ev))
    );
    assert_eq!(ev.redacts(RoomVersionId::V1), None);
    assert_eq!(ev.redacts(RoomVersionId::V11).unwrap(), "$nomore:example.com");

    assert_matches!(ev, RoomRedactionEvent::Original(OriginalRoomRedactionEvent::V11(v11_ev)));
    assert_eq!(v11_ev.content.redacts, "$nomore:example.com");
    assert_eq!(v11_ev.content.reason.as_deref(), Some("being very unfriendly"));
    assert_eq!(v11_ev.event_id, "$h29iv0s8:example.com");
    assert_eq!(v11_ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(v11_ev.room_id, "!roomid:room.com");
    assert_eq!(v11_ev.sender, "@carl:example.com");
    assert!(v11_ev.unsigned.is_empty());
}

#[test]
fn deserialize_redaction_v1_v11_compat() {
    let json_data = json!({
        "content": {
            "redacts": "$nomorev11:example.com",
            "reason": "being very unfriendly",
        },
        "redacts": "$nomorev1:example.com",
        "event_id": "$h29iv0s8:example.com",
        "sender": "@carl:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "type": "m.room.redaction",
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::RoomRedaction(ev))
    );
    assert_eq!(ev.redacts(RoomVersionId::V1).unwrap(), "$nomorev1:example.com");
    assert_eq!(ev.redacts(RoomVersionId::V11).unwrap(), "$nomorev11:example.com");

    assert_matches!(
        ev,
        RoomRedactionEvent::Original(OriginalRoomRedactionEvent::V1V11Compat(compat_ev))
    );
    assert_eq!(compat_ev.content.redacts, "$nomorev11:example.com");
    assert_eq!(compat_ev.content.reason.as_deref(), Some("being very unfriendly"));
    assert_eq!(compat_ev.redacts, "$nomorev1:example.com");
    assert_eq!(compat_ev.event_id, "$h29iv0s8:example.com");
    assert_eq!(compat_ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(compat_ev.room_id, "!roomid:room.com");
    assert_eq!(compat_ev.sender, "@carl:example.com");
    assert!(compat_ev.unsigned.is_empty());
}
