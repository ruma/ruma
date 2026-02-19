use ruma_common::{EventEncryptionAlgorithm, canonical_json::assert_to_canonical_json_eq, room_id};
use ruma_events::room_key::ToDeviceRoomKeyEventContent;
use serde_json::json;

#[test]
fn serialization() {
    let content = ToDeviceRoomKeyEventContent::new(
        EventEncryptionAlgorithm::MegolmV1AesSha2,
        room_id!("!testroomid:example.org"),
        "SessId".into(),
        "SessKey".into(),
    );

    assert_to_canonical_json_eq!(
        content,
        json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "room_id": "!testroomid:example.org",
            "session_id": "SessId",
            "session_key": "SessKey",
        })
    );
}
