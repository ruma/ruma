//! Types for the *m.room.encrypted* event.

use std::{collections::BTreeMap, time::SystemTime};

use js_int::UInt;
use ruma_identifiers::{DeviceId, EventId, RoomId, UserId};
use serde::{Deserialize, Serialize};

use crate::{EventType, FromRaw, UnsignedData};

/// This event type is used when sending encrypted events.
///
/// This type is to be used within a room. For a to-device event, use `EncryptedEventContent`
/// directly.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename = "m.room.encrypted")]
pub struct EncryptedEvent {
    /// The event's content.
    pub content: EncryptedEventContent,

    /// The unique identifier for the event.
    pub event_id: EventId,

    /// Time on originating homeserver when this event was sent.
    #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
    pub origin_server_ts: SystemTime,

    /// The unique identifier for the room associated with this event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<RoomId>,

    /// The unique identifier for the user who sent this event.
    pub sender: UserId,

    /// Additional key-value pairs not signed by the homeserver.
    #[serde(skip_serializing_if = "UnsignedData::is_empty")]
    pub unsigned: UnsignedData,
}

/// The payload for `EncryptedEvent`.
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
#[serde(tag = "algorithm")]
pub enum EncryptedEventContent {
    /// An event encrypted with *m.olm.v1.curve25519-aes-sha2*.
    #[serde(rename = "m.olm.v1.curve25519-aes-sha2")]
    OlmV1Curve25519AesSha2(OlmV1Curve25519AesSha2Content),

    /// An event encrypted with *m.megolm.v1.aes-sha2*.
    #[serde(rename = "m.megolm.v1.aes-sha2")]
    MegolmV1AesSha2(MegolmV1AesSha2Content),
}

impl FromRaw for EncryptedEvent {
    type Raw = raw::EncryptedEvent;

    fn from_raw(raw: raw::EncryptedEvent) -> Self {
        Self {
            content: FromRaw::from_raw(raw.content),
            event_id: raw.event_id,
            origin_server_ts: raw.origin_server_ts,
            room_id: raw.room_id,
            sender: raw.sender,
            unsigned: raw.unsigned,
        }
    }
}

impl FromRaw for EncryptedEventContent {
    type Raw = raw::EncryptedEventContent;

    fn from_raw(raw: raw::EncryptedEventContent) -> Self {
        use raw::EncryptedEventContent::*;

        match raw {
            OlmV1Curve25519AesSha2(content) => {
                EncryptedEventContent::OlmV1Curve25519AesSha2(content)
            }
            MegolmV1AesSha2(content) => EncryptedEventContent::MegolmV1AesSha2(content),
        }
    }
}

impl_room_event!(
    EncryptedEvent,
    EncryptedEventContent,
    EventType::RoomEncrypted
);

pub(crate) mod raw {
    use std::time::SystemTime;

    use ruma_identifiers::{EventId, RoomId, UserId};
    use serde::Deserialize;

    use super::{MegolmV1AesSha2Content, OlmV1Curve25519AesSha2Content};
    use crate::UnsignedData;

    /// This event type is used when sending encrypted events.
    ///
    /// This type is to be used within a room. For a to-device event, use `EncryptedEventContent`
    /// directly.
    #[derive(Clone, Debug, Deserialize)]
    pub struct EncryptedEvent {
        /// The event's content.
        pub content: EncryptedEventContent,

        /// The unique identifier for the event.
        pub event_id: EventId,

        /// Time on originating homeserver when this event was sent.
        #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
        pub origin_server_ts: SystemTime,

        /// The unique identifier for the room associated with this event.
        pub room_id: Option<RoomId>,

        /// The unique identifier for the user who sent this event.
        pub sender: UserId,

        /// Additional key-value pairs not signed by the homeserver.
        #[serde(default)]
        pub unsigned: UnsignedData,
    }

    /// The payload for `EncryptedEvent`.
    #[derive(Clone, Debug, Deserialize)]
    #[serde(tag = "algorithm")]
    pub enum EncryptedEventContent {
        /// An event encrypted with *m.olm.v1.curve25519-aes-sha2*.
        #[serde(rename = "m.olm.v1.curve25519-aes-sha2")]
        OlmV1Curve25519AesSha2(OlmV1Curve25519AesSha2Content),

        /// An event encrypted with *m.megolm.v1.aes-sha2*.
        #[serde(rename = "m.megolm.v1.aes-sha2")]
        MegolmV1AesSha2(MegolmV1AesSha2Content),
    }
}

/// The payload for `EncryptedEvent` using the *m.olm.v1.curve25519-aes-sha2* algorithm.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OlmV1Curve25519AesSha2Content {
    /// A map from the recipient Curve25519 identity key to ciphertext information.
    pub ciphertext: BTreeMap<String, CiphertextInfo>,

    /// The Curve25519 key of the sender.
    pub sender_key: String,
}

/// Ciphertext information holding the ciphertext and message type.
///
/// Used for messages encrypted with the *m.olm.v1.curve25519-aes-sha2* algorithm.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CiphertextInfo {
    /// The encrypted payload.
    pub body: String,

    /// The Olm message type.
    #[serde(rename = "type")]
    pub message_type: UInt,
}

/// The payload for `EncryptedEvent` using the *m.megolm.v1.aes-sha2* algorithm.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MegolmV1AesSha2Content {
    /// The encrypted content of the event.
    pub ciphertext: String,

    /// The Curve25519 key of the sender.
    pub sender_key: String,

    /// The ID of the sending device.
    pub device_id: DeviceId,

    /// The ID of the session used to encrypt the message.
    pub session_id: String,
}

#[cfg(test)]
mod tests {
    use matches::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{EncryptedEventContent, MegolmV1AesSha2Content};
    use crate::EventJson;

    #[test]
    fn serialization() {
        let key_verification_start_content =
            EncryptedEventContent::MegolmV1AesSha2(MegolmV1AesSha2Content {
                ciphertext: "ciphertext".to_string(),
                sender_key: "sender_key".to_string(),
                device_id: "device_id".to_string(),
                session_id: "session_id".to_string(),
            });

        let json_data = json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "ciphertext": "ciphertext",
            "sender_key": "sender_key",
            "device_id": "device_id",
            "session_id": "session_id"
        });

        assert_eq!(
            to_json_value(&key_verification_start_content).unwrap(),
            json_data
        );
    }

    #[test]
    fn deserialization() {
        let json_data = json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "ciphertext": "ciphertext",
            "sender_key": "sender_key",
            "device_id": "device_id",
            "session_id": "session_id"
        });

        assert_matches!(
            from_json_value::<EventJson<EncryptedEventContent>>(json_data)
                .unwrap()
                .deserialize()
                .unwrap(),
            EncryptedEventContent::MegolmV1AesSha2(MegolmV1AesSha2Content {
                ciphertext,
                sender_key,
                device_id,
                session_id,
            }) if ciphertext == "ciphertext"
                && sender_key == "sender_key"
                && device_id == "device_id"
                && session_id == "session_id"
        );
    }

    #[test]
    fn deserialization_olm() {
        let json_data = json!({
            "sender_key": "test_key",
            "ciphertext": {
                "test_curve_key": {
                    "body": "encrypted_body",
                    "type": 1
                }
            },
            "algorithm": "m.olm.v1.curve25519-aes-sha2"
        });
        let content = from_json_value::<EventJson<EncryptedEventContent>>(json_data)
            .unwrap()
            .deserialize()
            .unwrap();

        match content {
            EncryptedEventContent::OlmV1Curve25519AesSha2(c) => {
                assert_eq!(c.sender_key, "test_key");
                assert_eq!(c.ciphertext.len(), 1);
                assert_eq!(c.ciphertext["test_curve_key"].body, "encrypted_body");
                assert_eq!(c.ciphertext["test_curve_key"].message_type, 1u16.into());
            }
            _ => panic!("Wrong content type, expected a OlmV1 content"),
        }
    }

    #[test]
    fn deserialization_failure() {
        assert!(from_json_value::<EventJson<EncryptedEventContent>>(
            json!({ "algorithm": "m.megolm.v1.aes-sha2" })
        )
        .unwrap()
        .deserialize()
        .is_err());
    }
}
