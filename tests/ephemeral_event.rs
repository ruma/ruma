use std::{
    convert::TryFrom,
    time::{Duration, UNIX_EPOCH},
};

use maplit::btreemap;
use matches::assert_matches;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

use ruma_events::{
    receipt::{Receipt, ReceiptEventContent, Receipts},
    typing::TypingEventContent,
    AnyEphemeralRoomEventContent, EphemeralRoomEvent, EventJson,
};

#[test]
fn ephemeral_serialize_typing() {
    let aliases_event = EphemeralRoomEvent {
        content: AnyEphemeralRoomEventContent::Typing(TypingEventContent {
            user_ids: vec![UserId::try_from("@carl:example.com").unwrap()],
        }),
        room_id: RoomId::try_from("!roomid:room.com").unwrap(),
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

    assert_matches!(
        from_json_value::<EventJson<EphemeralRoomEvent<AnyEphemeralRoomEventContent>>>(json_data)
            .unwrap()
            .deserialize()
            .unwrap(),
        EphemeralRoomEvent {
            content: AnyEphemeralRoomEventContent::Typing(TypingEventContent {
                user_ids,
            }),
            room_id,
        } if user_ids[0] == UserId::try_from("@carl:example.com").unwrap()
            && room_id == RoomId::try_from("!roomid:room.com").unwrap()
    );
}

#[test]
fn ephemeral_serialize_receipt() {
    let event_id = EventId::try_from("$h29iv0s8:example.com").unwrap();
    let user_id = UserId::try_from("@carl:example.com").unwrap();

    let aliases_event = EphemeralRoomEvent {
        content: AnyEphemeralRoomEventContent::Receipt(ReceiptEventContent {
            receipts: btreemap! {
                event_id => Receipts {
                    read: Some(btreemap! {
                        user_id => Receipt { ts: Some(UNIX_EPOCH + Duration::from_millis(1)) },
                    }),
                },
            },
        }),
        room_id: RoomId::try_from("!roomid:room.com").unwrap(),
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
    let event_id = EventId::try_from("$h29iv0s8:example.com").unwrap();
    let user_id = UserId::try_from("@carl:example.com").unwrap();

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
        from_json_value::<EventJson<EphemeralRoomEvent<AnyEphemeralRoomEventContent>>>(json_data)
            .unwrap()
            .deserialize()
            .unwrap(),
        EphemeralRoomEvent {
            content: AnyEphemeralRoomEventContent::Receipt(ReceiptEventContent {
                receipts,
            }),
            room_id,
        } if !receipts.is_empty() && receipts.contains_key(&event_id)
            && room_id == RoomId::try_from("!roomid:room.com").unwrap()
            && receipts
                .get(&event_id)
                .map(|r| r.read.as_ref().unwrap().get(&user_id).unwrap().clone())
                .map(|r| r.ts)
                .unwrap()
                == Some(UNIX_EPOCH + Duration::from_millis(1))
    );
}
