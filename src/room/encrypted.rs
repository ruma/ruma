//! Types for the *m.room.encrypted* event.

use ruma_identifiers::DeviceId;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{from_value, Value};

use crate::Algorithm;

room_event! {
    /// This event type is used when sending encrypted events.
    ///
    /// This type is to be used within a room. For a to-device event, use `EncryptedEventContent`
    /// directly.
    pub struct EncryptedEvent(EncryptedEventContent) {}
}

/// The payload of an *m.room.encrypted* event.
#[derive(Clone, Debug, PartialEq)]
pub enum EncryptedEventContent {
    /// An event encrypted with *m.olm.v1.curve25519-aes-sha2*.
    OlmV1Curve25519AesSha2(OlmV1Curve25519AesSha2Content),

    /// An event encrypted with *m.megolm.v1.aes-sha2*.
    MegolmV1AesSha2(MegolmV1AesSha2Content),

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to ruma-events.
    #[doc(hidden)]
    __Nonexhaustive,
}

/// The payload of an *m.room.encrypted* event using the *m.olm.v1.curve25519-aes-sha2* algorithm.
#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
pub struct OlmV1Curve25519AesSha2Content {
    /// The encryption algorithm used to encrypt this event.
    pub algorithm: Algorithm,

    /// The encrypted content of the event.
    pub ciphertext: CiphertextInfo,

    /// The Curve25519 key of the sender.
    pub sender_key: String,
}

/// A map from the recipient Curve25519 identity key to ciphertext information.
///
/// Used for messages encrypted with the *m.olm.v1.curve25519-aes-sha2* algorithm.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CiphertextInfo {
    /// The encrypted payload.
    pub body: String,

    /// The Olm message type.
    #[serde(rename = "type")]
    pub message_type: u64,
}

/// The payload of an *m.room.encrypted* event using the *m.megolm.v1.aes-sha2* algorithm.
#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
pub struct MegolmV1AesSha2Content {
    /// The encryption algorithm used to encrypt this event.
    pub algorithm: Algorithm,

    /// The encrypted content of the event.
    pub ciphertext: String,

    /// The Curve25519 key of the sender.
    pub sender_key: String,

    /// The ID of the sending device.
    pub device_id: DeviceId,

    /// The ID of the session used to encrypt the message.
    pub session_id: String,
}

impl Serialize for EncryptedEventContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            EncryptedEventContent::OlmV1Curve25519AesSha2(ref content) => {
                content.serialize(serializer)
            }
            EncryptedEventContent::MegolmV1AesSha2(ref content) => content.serialize(serializer),
            _ => panic!("Attempted to serialize __Nonexhaustive variant."),
        }
    }
}

impl<'de> Deserialize<'de> for EncryptedEventContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Deserialize::deserialize(deserializer)?;

        let method_value = match value.get("algorithm") {
            Some(value) => value.clone(),
            None => return Err(D::Error::missing_field("algorithm")),
        };

        let method = match from_value::<Algorithm>(method_value.clone()) {
            Ok(method) => method,
            Err(error) => return Err(D::Error::custom(error.to_string())),
        };

        match method {
            Algorithm::OlmV1Curve25519AesSha2 => {
                let content = match from_value::<OlmV1Curve25519AesSha2Content>(value) {
                    Ok(content) => content,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(EncryptedEventContent::OlmV1Curve25519AesSha2(content))
            }
            Algorithm::MegolmV1AesSha2 => {
                let content = match from_value::<MegolmV1AesSha2Content>(value) {
                    Ok(content) => content,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(EncryptedEventContent::MegolmV1AesSha2(content))
            }
            Algorithm::Custom(_) => Err(D::Error::custom(
                "Custom algorithms are not supported by `EncryptedEventContent`.",
            )),
            Algorithm::__Nonexhaustive => Err(D::Error::custom(
                "Attempted to deserialize __Nonexhaustive variant.",
            )),
        }
    }
}
#[cfg(test)]
mod tests {
    use serde_json::{from_str, to_string};

    use super::{Algorithm, EncryptedEventContent, MegolmV1AesSha2Content};

    #[test]
    fn serializtion() {
        let key_verification_start_content =
            EncryptedEventContent::MegolmV1AesSha2(MegolmV1AesSha2Content {
                algorithm: Algorithm::MegolmV1AesSha2,
                ciphertext: "ciphertext".to_string(),
                sender_key: "sender_key".to_string(),
                device_id: "device_id".to_string(),
                session_id: "session_id".to_string(),
            });

        assert_eq!(
            to_string(&key_verification_start_content).unwrap(),
            r#"{"algorithm":"m.megolm.v1.aes-sha2","ciphertext":"ciphertext","sender_key":"sender_key","device_id":"device_id","session_id":"session_id"}"#
        );
    }

    #[test]
    fn deserialization() {
        let key_verification_start_content =
            EncryptedEventContent::MegolmV1AesSha2(MegolmV1AesSha2Content {
                algorithm: Algorithm::MegolmV1AesSha2,
                ciphertext: "ciphertext".to_string(),
                sender_key: "sender_key".to_string(),
                device_id: "device_id".to_string(),
                session_id: "session_id".to_string(),
            });

        assert_eq!(
            from_str::<EncryptedEventContent>(
                r#"{"algorithm":"m.megolm.v1.aes-sha2","ciphertext":"ciphertext","sender_key":"sender_key","device_id":"device_id","session_id":"session_id"}"#
            )
            .unwrap(),
            key_verification_start_content
        );
    }

    #[test]
    fn deserialization_failure() {
        assert!(
            from_str::<EncryptedEventContent>(r#"{"algorithm":"m.megolm.v1.aes-sha2"}"#).is_err()
        );
    }
}
