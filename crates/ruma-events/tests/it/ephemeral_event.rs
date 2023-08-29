use assert_matches2::assert_matches;
use js_int::uint;
use maplit::btreemap;
use ruma_common::{event_id, owned_event_id, owned_user_id, user_id, MilliSecondsSinceUnixEpoch};
use ruma_events::{
    receipt::{Receipt, ReceiptEventContent, ReceiptType},
    typing::TypingEventContent,
    AnyEphemeralRoomEvent,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn ephemeral_serialize_typing() {
    let content = TypingEventContent::new(vec![owned_user_id!("@carl:example.com")]);

    let actual = to_json_value(&content).unwrap();
    let expected = json!({
        "user_ids": ["@carl:example.com"],
    });

    assert_eq!(actual, expected);
}

#[test]
fn deserialize_ephemeral_typing() {
    let json_data = json!({
        "content": {
            "user_ids": [ "@carl:example.com" ]
        },
        "room_id": "!roomid:room.com",
        "type": "m.typing"
    });

    assert_matches!(
        from_json_value::<AnyEphemeralRoomEvent>(json_data),
        Ok(AnyEphemeralRoomEvent::Typing(typing_event))
    );
    assert_eq!(typing_event.content.user_ids.len(), 1);
    assert_eq!(typing_event.content.user_ids[0], "@carl:example.com");
    assert_eq!(typing_event.room_id, "!roomid:room.com");
}

#[test]
fn ephemeral_serialize_receipt() {
    let event_id = owned_event_id!("$h29iv0s8:example.com");
    let user_id = owned_user_id!("@carl:example.com");

    let content = ReceiptEventContent(btreemap! {
        event_id => btreemap! {
            ReceiptType::Read => btreemap! {
                user_id => Receipt::new(MilliSecondsSinceUnixEpoch(uint!(1))),
            },
        },
    });

    let actual = to_json_value(&content).unwrap();
    let expected = json!({
        "$h29iv0s8:example.com": {
            "m.read": {
                "@carl:example.com": { "ts": 1 }
            }
        }
    });

    assert_eq!(actual, expected);
}

#[test]
fn deserialize_ephemeral_receipt() {
    let event_id = event_id!("$h29iv0s8:example.com");
    let user_id = user_id!("@carl:example.com");

    let json_data = json!({
        "content": {
            "$h29iv0s8:example.com": {
                "m.read": {
                    "@carl:example.com": { "ts": 1 }
                }
            }
        },
        "room_id": "!roomid:room.com",
        "type": "m.receipt"
    });

    assert_matches!(
        from_json_value::<AnyEphemeralRoomEvent>(json_data),
        Ok(AnyEphemeralRoomEvent::Receipt(receipt_event))
    );
    let receipts = receipt_event.content.0;
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipt_event.room_id, "!roomid:room.com");
    let event_receipts = receipts.get(event_id).unwrap();
    let type_receipts = event_receipts.get(&ReceiptType::Read).unwrap();
    let user_receipt = type_receipts.get(user_id).unwrap();
    assert_eq!(user_receipt.ts, Some(MilliSecondsSinceUnixEpoch(uint!(1))));
}
