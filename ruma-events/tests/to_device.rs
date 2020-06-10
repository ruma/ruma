use std::convert::TryFrom;

use ruma_events::{
    room_key::RoomKeyEventContent, Algorithm, AnyToDeviceEventContent, ToDeviceEvent,
};
use ruma_identifiers::{RoomId, UserId};
use serde_json::{json, to_value as to_json_value};

#[test]
fn serialization() {
    let ev = ToDeviceEvent {
        sender: UserId::try_from("@example:example.org").unwrap(),
        content: AnyToDeviceEventContent::RoomKey(RoomKeyEventContent {
            algorithm: Algorithm::MegolmV1AesSha2,
            room_id: RoomId::try_from("!testroomid:example.org").unwrap(),
            session_id: "SessId".into(),
            session_key: "SessKey".into(),
        }),
    };

    assert_eq!(
        to_json_value(ev).unwrap(),
        json!({
            "type": "m.room_key",
            "sender": "@example:example.org",
            "content": {
                "algorithm": "m.megolm.v1.aes-sha2",
                "room_id": "!testroomid:example.org",
                "session_id": "SessId",
                "session_key": "SessKey",
            },
        })
    );
}
