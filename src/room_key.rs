//! Types for the *m.room_key* event.

use super::Algorithm;
use ruma_events_macros::BasicEventContent;
use ruma_identifiers::RoomId;
use serde::{Deserialize, Serialize};

/// This event type is used to exchange keys for end-to-end encryption.
///
/// Typically it is encrypted as an *m.room.encrypted* event, then sent as a to-device event.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.room_key")]
pub struct RoomKeyEventContent {
    /// The encryption algorithm the key in this event is to be used with.
    ///
    /// Must be `m.megolm.v1.aes-sha2`.
    pub algorithm: Algorithm,

    /// The room where the key is used.
    pub room_id: RoomId,

    /// The ID of the session that the key is for.
    pub session_id: String,

    /// The key to be exchanged.
    pub session_key: String,
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use ruma_identifiers::RoomId;
    use serde_json::{json, to_value as to_json_value};

    use super::RoomKeyEventContent;
    use crate::{Algorithm, BasicEvent};

    #[test]
    fn serialization() {
        let ev = BasicEvent {
            content: RoomKeyEventContent {
                algorithm: Algorithm::MegolmV1AesSha2,
                room_id: RoomId::try_from("!testroomid:example.org").unwrap(),
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
