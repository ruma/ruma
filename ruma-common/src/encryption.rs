//! Common types for [encryption] related tasks.
//!
//! [encryption]: https://matrix.org/docs/spec/client_server/r0.6.1#id76

use std::collections::BTreeMap;

use ruma_identifiers::{DeviceId, DeviceKeyId, EventEncryptionAlgorithm, UserId};
use serde::{Deserialize, Serialize};

/// Identity keys for a device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceKeys {
    /// The ID of the user the device belongs to. Must match the user ID used when logging in.
    pub user_id: UserId,

    /// The ID of the device these keys belong to. Must match the device ID used when logging in.
    pub device_id: Box<DeviceId>,

    /// The encryption algorithms supported by this device.
    pub algorithms: Vec<EventEncryptionAlgorithm>,

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
