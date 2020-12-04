//! Types for the *m.room.encrypted* event.

use std::collections::BTreeMap;

use js_int::UInt;
use ruma_events_macros::MessageEventContent;
use ruma_identifiers::DeviceIdBox;
use serde::{Deserialize, Serialize};

use crate::{room::message::Relation, MessageEvent};

/// An event that has been encrypted.
pub type EncryptedEvent = MessageEvent<EncryptedEventContent>;

/// The payload for `EncryptedEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, MessageEventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.encrypted")]
#[serde(tag = "algorithm")]
pub enum EncryptedEventContent {
    /// An event encrypted with *m.olm.v1.curve25519-aes-sha2*.
    #[serde(rename = "m.olm.v1.curve25519-aes-sha2")]
    OlmV1Curve25519AesSha2(OlmV1Curve25519AesSha2Content),

    /// An event encrypted with *m.megolm.v1.aes-sha2*.
    #[serde(rename = "m.megolm.v1.aes-sha2")]
    MegolmV1AesSha2(MegolmV1AesSha2Content),
}

/// The to-device version of the payload for the `EncryptedEvent`.
pub type EncryptedToDeviceEventContent = EncryptedEventContent;

/// The payload for `EncryptedEvent` using the *m.olm.v1.curve25519-aes-sha2* algorithm.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct OlmV1Curve25519AesSha2Content {
    /// A map from the recipient Curve25519 identity key to ciphertext information.
    pub ciphertext: BTreeMap<String, CiphertextInfo>,

    /// The Curve25519 key of the sender.
    pub sender_key: String,
}

impl OlmV1Curve25519AesSha2Content {
    /// Creates a new `OlmV1Curve25519AesSha2Content` with the given ciphertext and sender key.
    pub fn new(ciphertext: BTreeMap<String, CiphertextInfo>, sender_key: String) -> Self {
        Self { ciphertext, sender_key }
    }
}

/// Ciphertext information holding the ciphertext and message type.
///
/// Used for messages encrypted with the *m.olm.v1.curve25519-aes-sha2* algorithm.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct CiphertextInfo {
    /// The encrypted payload.
    pub body: String,

    /// The Olm message type.
    #[serde(rename = "type")]
    pub message_type: UInt,
}

impl CiphertextInfo {
    /// Creates a new `CiphertextInfo` with the given body and type.
    pub fn new(body: String, message_type: UInt) -> Self {
        Self { body, message_type }
    }
}

/// The payload for `EncryptedEvent` using the *m.megolm.v1.aes-sha2* algorithm.
///
/// To create an instance of this type, first create a `MegolmV1AesSha2ContentInit` and convert it
/// via `MegolmV1AesSha2Content::from` / `.into()`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct MegolmV1AesSha2Content {
    /// The encrypted content of the event.
    pub ciphertext: String,

    /// The Curve25519 key of the sender.
    pub sender_key: String,

    /// The ID of the sending device.
    pub device_id: DeviceIdBox,

    /// The ID of the session used to encrypt the message.
    pub session_id: String,

    /// Information about related messages for
    /// [rich replies](https://matrix.org/docs/spec/client_server/r0.6.1#rich-replies).
    #[serde(rename = "m.relates_to", skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Relation>,
}

/// Mandatory initial set of fields of `MegolmV1AesSha2Content`.
///
/// This struct will not be updated even if additional fields are added to `MegolmV1AesSha2Content`
/// in a new (non-breaking) release of the Matrix specification.
#[derive(Clone, Debug)]
pub struct MegolmV1AesSha2ContentInit {
    /// The encrypted content of the event.
    pub ciphertext: String,

    /// The Curve25519 key of the sender.
    pub sender_key: String,

    /// The ID of the sending device.
    pub device_id: DeviceIdBox,

    /// The ID of the session used to encrypt the message.
    pub session_id: String,
}

impl From<MegolmV1AesSha2ContentInit> for MegolmV1AesSha2Content {
    /// Creates a new `MegolmV1AesSha2Content` from the given init struct.
    fn from(init: MegolmV1AesSha2ContentInit) -> Self {
        let MegolmV1AesSha2ContentInit { ciphertext, sender_key, device_id, session_id } = init;
        Self { ciphertext, sender_key, device_id, session_id, relates_to: None }
    }
}

#[cfg(test)]
mod tests {
    use matches::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{EncryptedEventContent, MegolmV1AesSha2Content};
    use ruma_serde::Raw;

    #[test]
    fn serialization() {
        let key_verification_start_content =
            EncryptedEventContent::MegolmV1AesSha2(MegolmV1AesSha2Content {
                ciphertext: "ciphertext".into(),
                sender_key: "sender_key".into(),
                device_id: "device_id".into(),
                session_id: "session_id".into(),
                relates_to: None,
            });

        let json_data = json!({
            "algorithm": "m.megolm.v1.aes-sha2",
            "ciphertext": "ciphertext",
            "sender_key": "sender_key",
            "device_id": "device_id",
            "session_id": "session_id"
        });

        assert_eq!(to_json_value(&key_verification_start_content).unwrap(), json_data);
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
            from_json_value::<Raw<EncryptedEventContent>>(json_data)
                .unwrap()
                .deserialize()
                .unwrap(),
            EncryptedEventContent::MegolmV1AesSha2(MegolmV1AesSha2Content {
                ciphertext,
                sender_key,
                device_id,
                session_id,
                relates_to: None,
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
        let content = from_json_value::<Raw<EncryptedEventContent>>(json_data)
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
        assert!(from_json_value::<Raw<EncryptedEventContent>>(
            json!({ "algorithm": "m.megolm.v1.aes-sha2" })
        )
        .unwrap()
        .deserialize()
        .is_err());
    }
}
