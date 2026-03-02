//! Types for events used for secrets to be stored in the user's account_data.

use std::collections::BTreeMap;

use ruma_common::serde::{Base64, JsonCastable, Raw};
use serde::{Deserialize, Serialize};

/// A secret and its encrypted contents.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct SecretEventContent {
    /// Map from key ID to the encrypted data.
    ///
    /// The exact format for the encrypted data is dependent on the key algorithm.
    pub encrypted: BTreeMap<String, Raw<SecretEncryptedData>>,
}

impl SecretEventContent {
    /// Create a new `SecretEventContent` with the given encrypted content.
    pub fn new(encrypted: BTreeMap<String, Raw<SecretEncryptedData>>) -> Self {
        Self { encrypted }
    }
}

/// Encrypted data for a secret storage encryption algorithm.
///
/// This type cannot be constructed, it is only used for its semantic value and is meant to be used
/// with the `Raw::cast()` and `Raw::deserialize_as()` APIs.
///
/// It can be cast to or from the following types:
///
/// * [`AesHmacSha2EncryptedData`]
///
/// Convenience methods are also available for casting encrypted data from or to known compatible
/// types.
#[non_exhaustive]
pub struct SecretEncryptedData;

impl SecretEncryptedData {
    /// Construct a `Raw<SecretEncryptedData>` by casting the given serialized encrypted data.
    pub fn new<T: JsonCastable<Self>>(encrypted_data: Raw<T>) -> Raw<Self> {
        encrypted_data.cast()
    }

    /// Serialize the given encrypted data as a `Raw<SecretEncryptedData>`.
    pub fn serialize<T: Serialize + JsonCastable<Self>>(
        encrypted_data: &T,
    ) -> Result<Raw<Self>, serde_json::Error> {
        Raw::new(encrypted_data).map(Raw::cast)
    }

    /// Deserialize the given data encrypted with the `m.secret_storage.v1.aes-hmac-sha2` algorithm.
    pub fn deserialize_as_aes_hmac_sha2(
        encrypted_data: &Raw<Self>,
    ) -> Result<AesHmacSha2EncryptedData, serde_json::Error> {
        encrypted_data.deserialize_as()
    }
}

/// Data encrypted using the `m.secret_storage.v1.aes-hmac-sha2` algorithm.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct AesHmacSha2EncryptedData {
    /// The 16-byte initialization vector, encoded as base64.
    pub iv: Base64,

    /// The AES-CTR-encrypted data, encoded as base64.
    pub ciphertext: Base64,

    /// The MAC, encoded as base64.
    pub mac: Base64,
}

impl AesHmacSha2EncryptedData {
    /// Construct a new `` with the given initialization vector, ciphertext and MAC.
    pub fn new(iv: Base64, ciphertext: Base64, mac: Base64) -> Self {
        Self { iv, ciphertext, mac }
    }
}

impl JsonCastable<SecretEncryptedData> for AesHmacSha2EncryptedData {}

impl JsonCastable<AesHmacSha2EncryptedData> for SecretEncryptedData {}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use assert_matches2::assert_matches;
    use ruma_common::{canonical_json::assert_to_canonical_json_eq, serde::Base64};
    use serde_json::{from_value as from_json_value, json};

    use super::{AesHmacSha2EncryptedData, SecretEncryptedData, SecretEventContent};

    #[test]
    fn test_secret_serialization() {
        let key_one_data = AesHmacSha2EncryptedData {
            iv: Base64::parse("YWJjZGVmZ2hpamtsbW5vcA").unwrap(),
            ciphertext: Base64::parse("dGhpc2lzZGVmaW5pdGVseWNpcGhlcnRleHQ").unwrap(),
            mac: Base64::parse("aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U").unwrap(),
        };

        let mut encrypted = BTreeMap::new();
        encrypted
            .insert("key_one".to_owned(), SecretEncryptedData::serialize(&key_one_data).unwrap());

        let content = SecretEventContent::new(encrypted);

        assert_to_canonical_json_eq!(
            content,
            json!({
                "encrypted": {
                    "key_one" : {
                        "iv": "YWJjZGVmZ2hpamtsbW5vcA",
                        "ciphertext": "dGhpc2lzZGVmaW5pdGVseWNpcGhlcnRleHQ",
                        "mac": "aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U",
                    },
                },
            }),
        );
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
            SecretEncryptedData::deserialize_as_aes_hmac_sha2(secret_data),
            Ok(AesHmacSha2EncryptedData { iv, ciphertext, mac })
        );
        assert_eq!(iv.encode(), "YWJjZGVmZ2hpamtsbW5vcA");
        assert_eq!(ciphertext.encode(), "dGhpc2lzZGVmaW5pdGVseWNpcGhlcnRleHQ");
        assert_eq!(mac.encode(), "aWRvbnRrbm93d2hhdGFtYWNsb29rc2xpa2U");
    }
}
