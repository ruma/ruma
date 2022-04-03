//! Types for the *m.secret_storage.some_secret* event.

use std::collections::BTreeMap;

use ruma_common::events::macros::EventContent;
use serde::{Deserialize, Serialize};

/// The payload for `SomeSecretEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.secret_storage.some_secret", kind = GlobalAccountData)]
pub struct SomeSecretEventContent(BTreeMap<String, AesHmacSha2EncryptedData>);

/// Data encrypted using the *m.secret_storage.v1.aes-hmac-sha2* algorithm.
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AesHmacSha2EncryptedData {
    /// The 16-byte initialization vector, encoded as base64.
    pub iv: String,

    /// The AES-CTR-encrypted data, encoded as base64.
    pub ciphertext: String,

    /// The MAC, encoded as base64.
    pub mac: String,
}

impl AesHmacSha2EncryptedData {
    /// Creates a new `AesHmacSha2EncryptedData` with the given initialisation vector, ciphertext
    /// and MAC.
    pub fn new(iv: String, ciphertext: String, mac: String) -> Self {
        Self { iv, ciphertext, mac }
    }
}
