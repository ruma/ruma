//! Types for the [`m.secret_storage.key.*`] event.
//!
//! [`m.secret_storage.key.*`]: https://spec.matrix.org/v1.2/client-server-api/#key-storage

use js_int::{uint, UInt};
use serde::{Deserialize, Serialize};

use crate::{events::macros::EventContent, identifiers::KeyDerivationAlgorithm, serde::Base64};

/// A passphrase from which a key is to be derived.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PassPhrase {
    /// The algorithm to use to generate the key from the passphrase.
    ///
    /// Must be `m.pbkdf2`.
    pub algorithm: KeyDerivationAlgorithm,

    /// The salt used in PBKDF2.
    pub salt: String,

    /// The number of iterations to use in PBKDF2.
    pub iterations: UInt,

    /// The number of bits to generate for the key.
    ///
    /// Defaults to 256
    #[serde(default = "default_bits", skip_serializing_if = "is_default_bits")]
    pub bits: UInt,
}

impl PassPhrase {
    /// Creates a new `PassPhrase` with a given salt and number of iterations.
    pub fn new(salt: String, iterations: UInt) -> Self {
        Self { algorithm: KeyDerivationAlgorithm::Pbkfd2, salt, iterations, bits: default_bits() }
    }
}

fn default_bits() -> UInt {
    uint!(256)
}

fn is_default_bits(val: &UInt) -> bool {
    *val == default_bits()
}

/// A key description encrypted using a specified algorithm.
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[ruma_event(type = "m.secret_storage.key.*", kind = GlobalAccountData)]
pub struct SecretStorageKeyEventContent {
    /// The ID of the key.
    #[ruma_event(type_fragment)]
    #[serde(skip)]
    pub key_id: String,

    /// The name of the key.
    pub name: String,

    /// The encryption algorithm used for this key.
    ///
    /// Currently, only `m.secret_storage.v1.aes-hmac-sha2` is supported.
    #[serde(flatten)]
    pub algorithm: SecretEncryptionAlgorithm,

    /// The passphrase from which to generate the key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passphrase: Option<PassPhrase>,
}

impl SecretStorageKeyEventContent {
    /// Creates a `KeyDescription` with the given name.
    pub fn new(key_id: String, name: String, algorithm: SecretEncryptionAlgorithm) -> Self {
        Self { key_id, name, algorithm, passphrase: None }
    }
}

/// An algorithm and its properties, used to encrypt a secret.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "algorithm")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum SecretEncryptionAlgorithm {
    #[serde(rename = "m.secret_storage.v1.aes-hmac-sha2")]
    /// Encrypted using the `m.secrect_storage.v1.aes-hmac-sha2` algorithm.
    ///
    /// Secrets using this method are encrypted using AES-CTR-256 and authenticated using
    /// HMAC-SHA-256.
    SecretStorageV1AesHmacSha2 {
        /// The 16-byte initialization vector, encoded as base64.
        iv: Base64,

        /// The MAC, encoded as base64.
        mac: Base64,
    },
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use js_int::uint;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{PassPhrase, SecretEncryptionAlgorithm, SecretStorageKeyEventContent};
    use crate::{events::GlobalAccountDataEvent, serde::Base64, KeyDerivationAlgorithm};

    #[test]
    fn test_key_description_serialization() {
        let content = SecretStorageKeyEventContent::new(
            "my_key".into(),
            "my_key".into(),
            SecretEncryptionAlgorithm::SecretStorageV1AesHmacSha2 {
                iv: Base64::parse("YWJjZGVmZ2hpamtsbW5vcA").unwrap(),
                mac: Base64::parse("aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U").unwrap(),
            },
        );

        let json = json!({
            "name": "my_key",
            "algorithm": "m.secret_storage.v1.aes-hmac-sha2",
            "iv": "YWJjZGVmZ2hpamtsbW5vcA",
            "mac": "aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U"
        });

        assert_eq!(to_json_value(&content).unwrap(), json);
    }

    #[test]
    fn test_key_description_deserialization() {
        let json = json!({
            "name": "my_key",
            "algorithm": "m.secret_storage.v1.aes-hmac-sha2",
            "iv": "YWJjZGVmZ2hpamtsbW5vcA",
            "mac": "aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U"
        });

        assert_matches!(
            from_json_value(json).unwrap(),
            SecretStorageKeyEventContent {
                key_id: _,
                name,
                algorithm: SecretEncryptionAlgorithm::SecretStorageV1AesHmacSha2 {
                    iv,
                    mac,
                },
                passphrase: None,
            }
            if name == *"my_key"
                && iv == Base64::parse("YWJjZGVmZ2hpamtsbW5vcA").unwrap()
                && mac == Base64::parse("aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U").unwrap()
        )
    }

    #[test]
    fn test_key_description_with_passphrase_serialization() {
        let content = SecretStorageKeyEventContent {
            passphrase: Some(PassPhrase::new("rocksalt".into(), uint!(8))),
            ..SecretStorageKeyEventContent::new(
                "my_key".into(),
                "my_key".into(),
                SecretEncryptionAlgorithm::SecretStorageV1AesHmacSha2 {
                    iv: Base64::parse("YWJjZGVmZ2hpamtsbW5vcA").unwrap(),
                    mac: Base64::parse("aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U").unwrap(),
                },
            )
        };

        let json = json!({
            "name": "my_key",
            "algorithm": "m.secret_storage.v1.aes-hmac-sha2",
            "iv": "YWJjZGVmZ2hpamtsbW5vcA",
            "mac": "aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U",
            "passphrase": {
                "algorithm": "m.pbkdf2",
                "salt": "rocksalt",
                "iterations": 8
            }
        });

        assert_eq!(to_json_value(&content).unwrap(), json);
    }

    #[test]
    fn test_key_description_with_passphrase_deserialization() {
        let json = json!({
            "name": "my_key",
            "algorithm": "m.secret_storage.v1.aes-hmac-sha2",
            "iv": "YWJjZGVmZ2hpamtsbW5vcA",
            "mac": "aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U",
            "passphrase": {
                "algorithm": "m.pbkdf2",
                "salt": "rocksalt",
                "iterations": 8,
                "bits": 256
            }
        });

        assert_matches!(
            from_json_value(json).unwrap(),
            SecretStorageKeyEventContent {
                key_id: _key,
                name,
                algorithm: SecretEncryptionAlgorithm::SecretStorageV1AesHmacSha2 {
                    iv,
                    mac,
                },
                passphrase: Some(PassPhrase {
                    algorithm: KeyDerivationAlgorithm::Pbkfd2,
                    salt,
                    iterations,
                    bits
                })
            }
            if name == *"my_key"
                && iv == Base64::parse("YWJjZGVmZ2hpamtsbW5vcA").unwrap()
                && mac == Base64::parse("aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U").unwrap()
                && salt == *"rocksalt"
                && iterations == uint!(8)
                && bits == uint!(256)
        )
    }

    #[test]
    fn test_event_serialization() {
        let event = GlobalAccountDataEvent {
            content: SecretStorageKeyEventContent::new(
                "my_key_id".into(),
                "my_key".into(),
                SecretEncryptionAlgorithm::SecretStorageV1AesHmacSha2 {
                    iv: Base64::parse("YWJjZGVmZ2hpamtsbW5vcA").unwrap(),
                    mac: Base64::parse("aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U").unwrap(),
                },
            ),
        };

        let json = json!({
            "type": "m.secret_storage.key.my_key_id",
            "content": {
                "name": "my_key",
                "algorithm": "m.secret_storage.v1.aes-hmac-sha2",
                "iv": "YWJjZGVmZ2hpamtsbW5vcA",
                "mac": "aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U"
            }
        });

        assert_eq!(to_json_value(&event).unwrap(), json);
    }

    #[test]
    fn test_event_deserialization() {
        let json = json!({
            "type": "m.secret_storage.key.my_key_id",
            "content": {
                "name": "my_key",
                "algorithm": "m.secret_storage.v1.aes-hmac-sha2",
                "iv": "YWJjZGVmZ2hpamtsbW5vcA",
                "mac": "aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U"
            }
        });

        assert_matches!(
            from_json_value(json).unwrap(),
            GlobalAccountDataEvent {
                content: SecretStorageKeyEventContent {
                    key_id,
                    name,
                    algorithm: SecretEncryptionAlgorithm::SecretStorageV1AesHmacSha2 {
                        iv,
                        mac,
                    },
                    passphrase: None,
                }
            }
            if key_id == *"my_key_id"
                && name == *"my_key"
                && iv == Base64::parse("YWJjZGVmZ2hpamtsbW5vcA").unwrap()
                && mac == Base64::parse("aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U").unwrap()
        )
    }
}
