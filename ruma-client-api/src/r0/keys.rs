//! Endpoints for key management

use std::{collections::BTreeMap, fmt::Debug};

use ruma_events::Algorithm;
use ruma_identifiers::{DeviceId, DeviceKeyId, UserId};
use serde::{Deserialize, Serialize};

pub mod claim_keys;
pub mod get_key_changes;
pub mod get_keys;
pub mod upload_keys;

#[cfg(feature = "unstable-pre-spec")]
pub mod upload_signatures;
#[cfg(feature = "unstable-pre-spec")]
pub mod upload_signing_keys;

/// Identity keys for a device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceKeys {
    /// The ID of the user the device belongs to. Must match the user ID used when logging in.
    pub user_id: UserId,

    /// The ID of the device these keys belong to. Must match the device ID used when logging in.
    pub device_id: Box<DeviceId>,

    /// The encryption algorithms supported by this device.
    pub algorithms: Vec<Algorithm>,

    /// Public identity keys.
    pub keys: BTreeMap<DeviceKeyId, String>,

    /// Signatures for the device key object.
    pub signatures: BTreeMap<UserId, BTreeMap<DeviceKeyId, String>>,

    /// Additional data added to the device key information by intermediate servers, and
    /// not covered by the signatures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsigned: Option<UnsignedDeviceInfo>,
}

/// Additional data added to device key information by intermediate servers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsignedDeviceInfo {
    /// The display name which the user set on the device.
    pub device_display_name: Option<String>,
}

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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrossSigningKey {
    /// The ID of the user the key belongs to.
    pub user_id: UserId,
    /// What the key is used for.
    pub usage: Vec<KeyUsage>,
    /// The public key. The object must have exactly one property.
    pub keys: BTreeMap<String, String>,
    /// Signatures of the key. Only optional for master key.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub signatures: BTreeMap<UserId, BTreeMap<String, String>>,
}

/// The usage of a cross signing key.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyUsage {
    /// Master key.
    Master,
    /// Self-signing key.
    SelfSigning,
    /// User-signing key.
    UserSigning,
}
