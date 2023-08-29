//! Types for events used for secrets to be stored in the user's account_data.

use std::collections::BTreeMap;

use ruma_common::serde::Base64;
use serde::{Deserialize, Serialize};

/// A secret and its encrypted contents.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SecretEventContent {
    /// Map from key ID to the encrypted data.
    ///
    /// The exact format for the encrypted data is dependent on the key algorithm.
    pub encrypted: BTreeMap<String, SecretEncryptedData>,
}

impl SecretEventContent {
    /// Create a new `SecretEventContent` with the given encrypted content.
    pub fn new(encrypted: BTreeMap<String, SecretEncryptedData>) -> Self {
        Self { encrypted }
    }
}

/// Encrypted data for a corresponding secret storage encryption algorithm.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(untagged)]
pub enum SecretEncryptedData {
    /// Data encrypted using the *m.secret_storage.v1.aes-hmac-sha2* algorithm.
    AesHmacSha2EncryptedData {
        /// The 16-byte initialization vector, encoded as base64.
        iv: Base64,

        /// The AES-CTR-encrypted data, encoded as base64.
        ciphertext: Base64,

        /// The MAC, encoded as base64.
        mac: Base64,
    },
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use assert_matches2::assert_matches;
    use ruma_common::serde::Base64;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{SecretEncryptedData, SecretEventContent};

    #[test]
    fn test_secret_serialization() {
        let key_one_data = SecretEncryptedData::AesHmacSha2EncryptedData {
            iv: Base64::parse("YWJjZGVmZ2hpamtsbW5vcA").unwrap(),
            ciphertext: Base64::parse("dGhpc2lzZGVmaW5pdGVseWNpcGhlcnRleHQ").unwrap(),
            mac: Base64::parse("aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U").unwrap(),
        };

        let mut encrypted = BTreeMap::<String, SecretEncryptedData>::new();
        encrypted.insert("key_one".to_owned(), key_one_data);

        let content = SecretEventContent::new(encrypted);

        let json = json!({
            "encrypted": {
                "key_one" : {
                    "iv": "YWJjZGVmZ2hpamtsbW5vcA",
                    "ciphertext": "dGhpc2lzZGVmaW5pdGVseWNpcGhlcnRleHQ",
                    "mac": "aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U"
                }
            }
        });

        assert_eq!(to_json_value(content).unwrap(), json);
    }

    #[test]
    fn test_secret_deserialization() {
        let json = json!({
            "encrypted": {
                "key_one" : {
                    "iv": "YWJjZGVmZ2hpamtsbW5vcA",
                    "ciphertext": "dGhpc2lzZGVmaW5pdGVseWNpcGhlcnRleHQ",
                    "mac": "aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U"
                }
            }
        });

        let deserialized: SecretEventContent = from_json_value(json).unwrap();
        let secret_data = deserialized.encrypted.get("key_one").unwrap();

        assert_matches!(
            secret_data,
            SecretEncryptedData::AesHmacSha2EncryptedData { iv, ciphertext, mac }
        );
        assert_eq!(iv.encode(), "YWJjZGVmZ2hpamtsbW5vcA");
        assert_eq!(ciphertext.encode(), "dGhpc2lzZGVmaW5pdGVseWNpcGhlcnRleHQ");
        assert_eq!(mac.encode(), "aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U");
    }
}
