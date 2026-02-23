//! Types for the [`m.room_key`] event.
//!
//! [`m.room_key`]: https://spec.matrix.org/latest/client-server-api/#mroom_key

use ruma_common::{EventEncryptionAlgorithm, RoomId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

pub mod withheld;

/// The content of an `m.room_key` event.
///
/// Typically encrypted as an `m.room.encrypted` event, then sent as a to-device event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.room_key", kind = ToDevice)]
pub struct ToDeviceRoomKeyEventContent {
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

    /// Used to mark key if allowed for shared history.
    ///
    /// Defaults to `false`.
    #[cfg(feature = "unstable-msc3061")]
    #[serde(
        default,
        rename = "org.matrix.msc3061.shared_history",
        skip_serializing_if = "ruma_common::serde::is_default"
    )]
    pub shared_history: bool,
}

impl ToDeviceRoomKeyEventContent {
    /// Creates a new `ToDeviceRoomKeyEventContent` with the given algorithm, room ID, session ID
    /// and session key.
    pub fn new(
        algorithm: EventEncryptionAlgorithm,
        room_id: RoomId,
        session_id: String,
        session_key: String,
    ) -> Self {
        Self {
            algorithm,
            room_id,
            session_id,
            session_key,
            #[cfg(feature = "unstable-msc3061")]
            shared_history: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::{canonical_json::assert_to_canonical_json_eq, owned_room_id};
    use serde_json::json;

    use super::ToDeviceRoomKeyEventContent;
    use crate::EventEncryptionAlgorithm;

    #[test]
    fn serialization() {
        let content = ToDeviceRoomKeyEventContent {
            algorithm: EventEncryptionAlgorithm::MegolmV1AesSha2,
            room_id: owned_room_id!("!testroomid:example.org"),
            session_id: "SessId".into(),
            session_key: "SessKey".into(),
            #[cfg(feature = "unstable-msc3061")]
            shared_history: true,
        };

        #[cfg(not(feature = "unstable-msc3061"))]
        assert_to_canonical_json_eq!(
            content,
            json!({
                "algorithm": "m.megolm.v1.aes-sha2",
                "room_id": "!testroomid:example.org",
                "session_id": "SessId",
                "session_key": "SessKey",
            })
        );

        #[cfg(feature = "unstable-msc3061")]
        assert_to_canonical_json_eq!(
            content,
            json!({
                "algorithm": "m.megolm.v1.aes-sha2",
                "room_id": "!testroomid:example.org",
                "session_id": "SessId",
                "session_key": "SessKey",
                "org.matrix.msc3061.shared_history": true,
            })
        );
    }
}
