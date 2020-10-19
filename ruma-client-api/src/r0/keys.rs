//! Endpoints for key management

use std::{collections::BTreeMap, fmt::Debug};

use ruma_identifiers::{DeviceKeyId, UserId};
use serde::{Deserialize, Serialize};

pub mod claim_keys;
pub mod get_key_changes;
pub mod get_keys;
pub mod upload_keys;

#[cfg(feature = "unstable-pre-spec")]
pub mod upload_signatures;
#[cfg(feature = "unstable-pre-spec")]
pub mod upload_signing_keys;

/// A key for the SignedCurve25519 algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedKey {
    /// Base64-encoded 32-byte Curve25519 public key.
    pub key: String,

    /// Signatures for the key object.
    pub signatures: BTreeMap<UserId, BTreeMap<DeviceKeyId, String>>,
}

/// A one-time public key for "pre-key" messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneTimeKey {
    /// A key containing signatures, for the SignedCurve25519 algorithm.
    SignedKey(SignedKey),

    /// A string-valued key, for the Ed25519 and Curve25519 algorithms.
    Key(String),
}

/// A cross signing key.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CrossSigningKey {
    /// The ID of the user the key belongs to.
    pub user_id: UserId,

    /// What the key is used for.
    pub usage: Vec<KeyUsage>,

    /// The public key. The object must have exactly one property.
    pub keys: BTreeMap<String, String>,

    /// Signatures of the key. Only optional for master key.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub signatures: BTreeMap<UserId, BTreeMap<String, String>>,
}

/// The usage of a cross signing key.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyUsage {
    /// Master key.
    Master,

    /// Self-signing key.
    SelfSigning,

    /// User-signing key.
    UserSigning,
}
