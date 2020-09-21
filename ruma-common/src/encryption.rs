//! Common types for [encryption] related tasks.
//!
//! [encryption]: https://matrix.org/docs/spec/client_server/r0.6.1#id76

use std::collections::BTreeMap;

use ruma_identifiers::{DeviceIdBox, DeviceKeyId, EventEncryptionAlgorithm, UserId};
use serde::{Deserialize, Serialize};

/// Identity keys for a device.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct DeviceKeys {
    /// The ID of the user the device belongs to. Must match the user ID used when logging in.
    pub user_id: UserId,

    /// The ID of the device these keys belong to. Must match the device ID used when logging in.
    pub device_id: DeviceIdBox,

    /// The encryption algorithms supported by this device.
    pub algorithms: Vec<EventEncryptionAlgorithm>,

    /// Public identity keys.
    pub keys: BTreeMap<DeviceKeyId, String>,

    /// Signatures for the device key object.
    pub signatures: BTreeMap<UserId, BTreeMap<DeviceKeyId, String>>,

    /// Additional data added to the device key information by intermediate servers, and
    /// not covered by the signatures.
    #[serde(default, skip_serializing_if = "UnsignedDeviceInfo::is_empty")]
    pub unsigned: UnsignedDeviceInfo,
}

impl DeviceKeys {
    /// Creates a new `DeviceKeys` from the given user id, device id, algorithms, keys and
    /// signatures.
    pub fn new(
        user_id: UserId,
        device_id: DeviceIdBox,
        algorithms: Vec<EventEncryptionAlgorithm>,
        keys: BTreeMap<DeviceKeyId, String>,
        signatures: BTreeMap<UserId, BTreeMap<DeviceKeyId, String>>,
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
