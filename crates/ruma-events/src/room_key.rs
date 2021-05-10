//! Types for the *m.room_key* event.

use ruma_events_macros::BasicEventContent;
use ruma_identifiers::{EventEncryptionAlgorithm, RoomId};
use serde::{Deserialize, Serialize};

/// The payload for `RoomKeyEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.room_key")]
pub struct RoomKeyToDeviceEventContent {
    /// The encryption algorithm the key in this event is to be used with.
    ///
    /// Must be `m.megolm.v1.aes-sha2`.
    pub algorithm: EventEncryptionAlgorithm,

    /// The room where the key is used.
    pub room_id: RoomId,

    /// The ID of the session that the key is for.
    pub session_id: String,

    /// The key to be exchanged.
    pub session_key: String,
}

#[cfg(test)]
mod tests {
    use ruma_identifiers::{room_id, user_id, EventEncryptionAlgorithm};
    use serde_json::{json, to_value as to_json_value};

    use super::RoomKeyToDeviceEventContent;
    use crate::ToDeviceEvent;

    #[test]
    fn serialization() {
        let ev = ToDeviceEvent {
            content: RoomKeyToDeviceEventContent {
                algorithm: EventEncryptionAlgorithm::MegolmV1AesSha2,
                room_id: room_id!("!testroomid:example.org"),
                session_id: "SessId".into(),
                session_key: "SessKey".into(),
            },
            sender: user_id!("@user:example.org"),
        };

        assert_eq!(
            to_json_value(ev).unwrap(),
            json!({
                "type": "m.room_key",
                "content": {
                    "algorithm": "m.megolm.v1.aes-sha2",
                    "room_id": "!testroomid:example.org",
                    "session_id": "SessId",
                    "session_key": "SessKey",
                },
                "sender": "@user:example.org",
            })
        );
    }
}
