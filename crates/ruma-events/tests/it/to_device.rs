use ruma_common::{owned_room_id, EventEncryptionAlgorithm};
use ruma_events::room_key::ToDeviceRoomKeyEventContent;
use serde_json::{json, to_value as to_json_value};

#[test]
fn serialization() {
    let content = ToDeviceRoomKeyEventContent::new(
        EventEncryptionAlgorithm::MegolmV1AesSha2,
        owned_room_id!("!testroomid:example.org"),
        "SessId".into(),
        "SessKey".into(),
        #[cfg(feature = "unstable-msc3061")]
        true,
    );

    #[cfg(not(feature = "unstable-msc3061"))]
    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "room_id": "!testroomid:example.org",
            "session_id": "SessId",
            "session_key": "SessKey",
        })
    );

    #[cfg(feature = "unstable-msc3061")]
    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "room_id": "!testroomid:example.org",
            "session_id": "SessId",
            "session_key": "SessKey",
            "org.matrix.msc3061.shared_history": true,
        })
    );
}
