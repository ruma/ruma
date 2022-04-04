//! Types for the [`m.secret_storage.key.*`] event.
//!
//! [`m.secret_storage.key.*`]: https://spec.matrix.org/v1.2/client-server-api/#key-storage

use js_int::{uint, UInt};
use serde::{Deserialize, Serialize};

use crate::{
    events::macros::EventContent,
    identifiers::{KeyDerivationAlgorithm, SecretEncryptionAlgorithm},
};

/// The payload for `KeyEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.secret_storage.key.*", kind = GlobalAccountData)]
pub struct KeyEventContent {
    /// The ID of the key.
    #[ruma_event(type_fragment)]
    pub key_id: String,

    /// The name of the key.
    ///
    /// If not given, the client may use a generic name such as "Unnamed key", or "Default Key" if
    /// the key is marked as the default key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The encryption algorithm to be used for this key.
    ///
    /// Currently, only `m.secret_storage.v1.aes-hmac-sha2` is supported.
    pub algorithm: SecretEncryptionAlgorithm,

    /// The passphrase from which to generate the key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passphrase: Option<PassPhrase>,
}

impl KeyEventContent {
    /// Creates a new KeyEventContent with the given key ID, no name and no passphrase.
    pub fn new(key_id: String) -> Self {
        Self {
            key_id,
            name: None,
            algorithm: SecretEncryptionAlgorithm::SecretStorageV1AesHmacSha2,
            passphrase: None,
        }
    }
}

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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyDescription {
    /// The name of the key.
    pub name: String,

    /// The encryption algorithm used for this key.
    ///
    /// Currently, only `m.secret_storage.v1.aes-hmac-sha2` is supported.
    pub algorithm: SecretEncryptionAlgorithm,

    /// The passphrase from which to generate the key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passphrase: Option<PassPhrase>,

    /// The 16-byte initialization vector, encoded as base64.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iv: Option<String>,

    /// The MAC of the result of encrypting 32 bytes of 0, encoded as base64.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac: Option<String>,
}

impl KeyDescription {
    /// Creates a `KeyDescription` with the given name.
    pub fn new(name: String) -> Self {
        Self {
            name,
            algorithm: SecretEncryptionAlgorithm::SecretStorageV1AesHmacSha2,
            passphrase: None,
            iv: None,
            mac: None,
        }
    }
}
