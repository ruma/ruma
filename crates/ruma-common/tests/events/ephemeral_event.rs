use assert_matches::assert_matches;
use js_int::uint;
use maplit::btreemap;
use ruma_common::{event_id, receipt::ReceiptType, room_id, user_id, MilliSecondsSinceUnixEpoch};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

use ruma_common::events::{
    receipt::{Receipt, ReceiptEventContent},
    typing::TypingEventContent,
    AnyEphemeralRoomEvent, EphemeralRoomEvent,
};

#[test]
fn ephemeral_serialize_typing() {
    let aliases_event = EphemeralRoomEvent {
        content: TypingEventContent::new(vec![user_id!("@carl:example.com").to_owned()]),
        room_id: room_id!("!roomid:room.com").to_owned(),
    };

    let actual = to_json_value(&aliases_event).unwrap();
    let expected = json!({
        "content": {
            "user_ids": [ "@carl:example.com" ]
        },
        "room_id": "!roomid:room.com",
        "type": "m.typing",
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

    let typing_event = assert_matches!(
        from_json_value::<AnyEphemeralRoomEvent>(json_data),
        Ok(AnyEphemeralRoomEvent::Typing(typing_event)) => typing_event
    );
    assert_eq!(typing_event.content.user_ids[0], "@carl:example.com");
    assert_eq!(typing_event.room_id, "!roomid:room.com");
}

#[test]
fn ephemeral_serialize_receipt() {
    let event_id = event_id!("$h29iv0s8:example.com").to_owned();
    let user_id = user_id!("@carl:example.com").to_owned();

    let aliases_event = EphemeralRoomEvent {
        content: ReceiptEventContent(btreemap! {
            event_id => btreemap! {
                ReceiptType::Read => btreemap! {
                    user_id => Receipt::new(MilliSecondsSinceUnixEpoch(uint!(1))),
                },
            },
        }),
        room_id: room_id!("!roomid:room.com").to_owned(),
    };

    let actual = to_json_value(&aliases_event).unwrap();
    let expected = json!({
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

    let receipt_event = assert_matches!(
        from_json_value::<AnyEphemeralRoomEvent>(json_data),
        Ok(AnyEphemeralRoomEvent::Receipt(receipt_event)) => receipt_event
    );
    let receipts = receipt_event.content.0;
    assert!(!receipts.is_empty());
    assert!(receipts.contains_key(event_id));
    assert_eq!(receipt_event.room_id, "!roomid:room.com");
    let event_receipts = assert_matches!(receipts.get(event_id), Some(r) => r);
    let type_receipts = assert_matches!(event_receipts.get(&ReceiptType::Read), Some(r) => r);
    let user_receipt = assert_matches!(type_receipts.get(user_id), Some(r) => r);
    assert_eq!(user_receipt.ts, Some(MilliSecondsSinceUnixEpoch(uint!(1))));
}
