//! Types for the *m.room_key* event.

use ruma_events_macros::BasicEventContent;
use ruma_identifiers::{EventEncryptionAlgorithm, RoomId};
use serde::{Deserialize, Serialize};

use crate::BasicEvent;

/// Typically encrypted as an *m.room.encrypted* event, then sent as a to-device event.
pub type RoomKeyEvent = BasicEvent<RoomKeyEventContent>;

/// The payload for `RoomKeyEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.room_key")]
pub struct RoomKeyEventContent {
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

/// The to-device version of the payload for the `RoomKeyEvent`.
pub type RoomKeyToDeviceEventContent = RoomKeyEventContent;

#[cfg(test)]
mod tests {
    use ruma_identifiers::{room_id, EventEncryptionAlgorithm};
    use serde_json::{json, to_value as to_json_value};

    use super::RoomKeyEventContent;
    use crate::BasicEvent;

    #[test]
    fn serialization() {
        let ev = BasicEvent {
            content: RoomKeyEventContent {
                algorithm: EventEncryptionAlgorithm::MegolmV1AesSha2,
                room_id: room_id!("!testroomid:example.org"),
                session_id: "SessId".into(),
                session_key: "SessKey".into(),
            },
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
            })
        );
    }
}
