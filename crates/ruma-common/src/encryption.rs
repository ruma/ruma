//! Common types for [encryption] related tasks.
//!
//! [encryption]: https://spec.matrix.org/latest/client-server-api/#end-to-end-encryption

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    serde::{Base64, StringEnum},
    EventEncryptionAlgorithm, OwnedDeviceId, OwnedDeviceKeyId, OwnedUserId, PrivOwnedStr,
};

/// Identity keys for a device.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct DeviceKeys {
    /// The ID of the user the device belongs to.
    ///
    /// Must match the user ID used when logging in.
    pub user_id: OwnedUserId,

    /// The ID of the device these keys belong to.
    ///
    /// Must match the device ID used when logging in.
    pub device_id: OwnedDeviceId,

    /// The encryption algorithms supported by this device.
    pub algorithms: Vec<EventEncryptionAlgorithm>,

    /// Public identity keys.
    pub keys: BTreeMap<OwnedDeviceKeyId, String>,

    /// Signatures for the device key object.
    pub signatures: BTreeMap<OwnedUserId, BTreeMap<OwnedDeviceKeyId, String>>,

    /// Additional data added to the device key information by intermediate servers, and
    /// not covered by the signatures.
    #[serde(default, skip_serializing_if = "UnsignedDeviceInfo::is_empty")]
    pub unsigned: UnsignedDeviceInfo,
}

impl DeviceKeys {
    /// Creates a new `DeviceKeys` from the given user id, device id, algorithms, keys and
    /// signatures.
    pub fn new(
        user_id: OwnedUserId,
        device_id: OwnedDeviceId,
        algorithms: Vec<EventEncryptionAlgorithm>,
        keys: BTreeMap<OwnedDeviceKeyId, String>,
        signatures: BTreeMap<OwnedUserId, BTreeMap<OwnedDeviceKeyId, String>>,
    ) -> Self {
        Self { user_id, device_id, algorithms, keys, signatures, unsigned: Default::default() }
    }
}

/// Additional data added to device key information by intermediate servers.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct UnsignedDeviceInfo {
    /// The display name which the user set on the device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_display_name: Option<String>,
}

impl UnsignedDeviceInfo {
    /// Creates an empty `UnsignedDeviceInfo`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Checks whether all fields are empty / `None`.
    pub fn is_empty(&self) -> bool {
        self.device_display_name.is_none()
    }
}

/// Signatures for a `SignedKey` object.
pub type SignedKeySignatures = BTreeMap<OwnedUserId, BTreeMap<OwnedDeviceKeyId, String>>;

/// A key for the SignedCurve25519 algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SignedKey {
    /// Base64-encoded 32-byte Curve25519 public key.
    pub key: Base64,

    /// Signatures for the key object.
    pub signatures: SignedKeySignatures,

    /// Is this key considered to be a fallback key, defaults to false.
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub fallback: bool,
}

impl SignedKey {
    /// Creates a new `SignedKey` with the given key and signatures.
    pub fn new(key: Base64, signatures: SignedKeySignatures) -> Self {
        Self { key, signatures, fallback: false }
    }

    /// Creates a new fallback `SignedKey` with the given key and signatures.
    pub fn new_fallback(key: Base64, signatures: SignedKeySignatures) -> Self {
        Self { key, signatures, fallback: true }
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
pub type CrossSigningKeySignatures = BTreeMap<OwnedUserId, BTreeMap<OwnedDeviceKeyId, String>>;

/// A cross signing key.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct CrossSigningKey {
    /// The ID of the user the key belongs to.
    pub user_id: OwnedUserId,

    /// What the key is used for.
    pub usage: Vec<KeyUsage>,

    /// The public key.
    ///
    /// The object must have exactly one property.
    pub keys: BTreeMap<OwnedDeviceKeyId, String>,

    /// Signatures of the key.
    ///
    /// Only optional for master key.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub signatures: CrossSigningKeySignatures,
}

impl CrossSigningKey {
    /// Creates a new `CrossSigningKey` with the given user ID, usage, keys and signatures.
    pub fn new(
        user_id: OwnedUserId,
        usage: Vec<KeyUsage>,
        keys: BTreeMap<OwnedDeviceKeyId, String>,
        signatures: CrossSigningKeySignatures,
    ) -> Self {
        Self { user_id, usage, keys, signatures }
    }
}

/// The usage of a cross signing key.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, StringEnum)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_enum(rename_all = "snake_case")]
pub enum KeyUsage {
    /// Master key.
    Master,

    /// Self-signing key.
    SelfSigning,

    /// User-signing key.
    UserSigning,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
