//! Endpoints for key management

use std::{collections::BTreeMap, fmt::Debug};

use ruma_identifiers::{DeviceKeyId, UserId};
use serde::{Deserialize, Serialize};

pub mod claim_keys;
pub mod get_key_changes;
pub mod get_keys;
pub mod upload_keys;

#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
pub mod upload_signatures;
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
pub mod upload_signing_keys;

/// Signatures for a `SignedKey` object.
pub type SignedKeySignatures = BTreeMap<UserId, BTreeMap<DeviceKeyId, String>>;

/// A key for the SignedCurve25519 algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SignedKey {
    /// Base64-encoded 32-byte Curve25519 public key.
    pub key: String,

    /// Signatures for the key object.
    pub signatures: SignedKeySignatures,
}

impl SignedKey {
    /// Creates a new `SignedKey` with the given key and signatures.
    pub fn new(key: String, signatures: SignedKeySignatures) -> Self {
        Self { key, signatures }
    }
}

/// A one-time public key for "pre-key" messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(untagged)]
pub enum OneTimeKey {
    /// A key containing signatures, for the SignedCurve25519 algorithm.
    SignedKey(SignedKey),

    /// A string-valued key, for the Ed25519 and Curve25519 algorithms.
    Key(String),
}

/// Signatures for a `CrossSigningKey` object.
pub type CrossSigningKeySignatures = BTreeMap<UserId, BTreeMap<String, String>>;

/// A cross signing key.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct CrossSigningKey {
    /// The ID of the user the key belongs to.
    pub user_id: UserId,

    /// What the key is used for.
    pub usage: Vec<KeyUsage>,

    /// The public key. The object must have exactly one property.
    pub keys: BTreeMap<String, String>,

    /// Signatures of the key. Only optional for master key.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub signatures: CrossSigningKeySignatures,
}

impl CrossSigningKey {
    /// Creates a new `CrossSigningKey` with the given user ID, usage, keys and
    /// signatures.
    pub fn new(
        user_id: UserId,
        usage: Vec<KeyUsage>,
        keys: BTreeMap<String, String>,
        signatures: CrossSigningKeySignatures,
    ) -> Self {
        Self { user_id, usage, keys, signatures }
    }
}

/// The usage of a cross signing key.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(rename_all = "snake_case")]
pub enum KeyUsage {
    /// Master key.
    Master,

    /// Self-signing key.
    SelfSigning,

    /// User-signing key.
    UserSigning,
}
