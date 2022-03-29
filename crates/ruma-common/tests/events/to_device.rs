use ruma_common::{
    events::{room_key::ToDeviceRoomKeyEventContent, ToDeviceEvent},
    room_id, user_id, EventEncryptionAlgorithm,
};
use serde_json::{json, to_value as to_json_value};

#[test]
fn serialization() {
    let ev = ToDeviceEvent {
        sender: user_id!("@example:example.org").to_owned(),
        content: ToDeviceRoomKeyEventContent::new(
            EventEncryptionAlgorithm::MegolmV1AesSha2,
            room_id!("!testroomid:example.org").to_owned(),
            "SessId".into(),
            "SessKey".into(),
        ),
    };

    assert_eq!(
        to_json_value(&ev).unwrap(),
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
