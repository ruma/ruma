//! Types for the [`m.room_key`] event.
//!
//! [`m.room_key`]: https://spec.matrix.org/v1.2/client-server-api/#mroom_key

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{EventEncryptionAlgorithm, OwnedRoomId};

/// The content of an `m.room_key` event.
///
/// Typically encrypted as an `m.room.encrypted` event, then sent as a to-device event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room_key", kind = ToDevice)]
pub struct ToDeviceRoomKeyEventContent {
    /// The encryption algorithm the key in this event is to be used with.
    ///
    /// Must be `m.megolm.v1.aes-sha2`.
    pub algorithm: EventEncryptionAlgorithm,

    /// The room where the key is used.
    pub room_id: OwnedRoomId,

    /// The ID of the session that the key is for.
    pub session_id: String,

    /// The key to be exchanged.
    pub session_key: String,
}

impl ToDeviceRoomKeyEventContent {
    /// Creates a new `ToDeviceRoomKeyEventContent` with the given algorithm, room ID, session ID
    /// and session key.
    pub fn new(
        algorithm: EventEncryptionAlgorithm,
        room_id: OwnedRoomId,
        session_id: String,
        session_key: String,
    ) -> Self {
        Self { algorithm, room_id, session_id, session_key }
    }
}

#[cfg(test)]
mod tests {
    use crate::{room_id, user_id, EventEncryptionAlgorithm};
    use serde_json::{json, to_value as to_json_value};

    use super::ToDeviceRoomKeyEventContent;
    use crate::events::ToDeviceEvent;

    #[test]
    fn serialization() {
        let ev = ToDeviceEvent {
            content: ToDeviceRoomKeyEventContent {
                algorithm: EventEncryptionAlgorithm::MegolmV1AesSha2,
                room_id: room_id!("!testroomid:example.org").to_owned(),
                session_id: "SessId".into(),
                session_key: "SessKey".into(),
            },
            sender: user_id!("@user:example.org").to_owned(),
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
