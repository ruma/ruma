//! Common types for [encryption] related tasks.
//!
//! [encryption]: https://matrix.org/docs/spec/client_server/r0.6.1#id76

use std::collections::BTreeMap;

use ruma_api::Outgoing;
use ruma_identifiers::{DeviceId, DeviceKeyId, EventEncryptionAlgorithm, UserId};
use ruma_serde::CanBeEmpty;
use serde::Serialize;

/// Identity keys for a device.
#[derive(Clone, Debug, Outgoing, Serialize)]
#[non_exhaustive]
#[incoming_derive(Clone, Serialize)]
pub struct DeviceKeys<'a> {
    /// The ID of the user the device belongs to. Must match the user ID used when logging in.
    pub user_id: &'a UserId,

    /// The ID of the device these keys belong to. Must match the device ID used when logging in.
    pub device_id: &'a DeviceId,

    /// The encryption algorithms supported by this device.
    pub algorithms: &'a [EventEncryptionAlgorithm],

    /// Public identity keys.
    pub keys: BTreeMap<DeviceKeyId, String>,

    /// Signatures for the device key object.
    pub signatures: BTreeMap<UserId, BTreeMap<DeviceKeyId, String>>,

    /// Additional data added to the device key information by intermediate servers, and
    /// not covered by the signatures.
    #[serde(skip_serializing_if = "ruma_serde::is_empty")]
    pub unsigned: UnsignedDeviceInfo<'a>,
}

impl<'a> DeviceKeys<'a> {
    /// Creates a new `DeviceKeys` from the given user id, device id, algorithms, keys and
    /// signatures.
    pub fn new(
        user_id: &'a UserId,
        device_id: &'a DeviceId,
        algorithms: &'a [EventEncryptionAlgorithm],
        keys: BTreeMap<DeviceKeyId, String>,
        signatures: BTreeMap<UserId, BTreeMap<DeviceKeyId, String>>,
    ) -> Self {
        Self { user_id, device_id, algorithms, keys, signatures, unsigned: Default::default() }
    }
}

/// Additional data added to device key information by intermediate servers.
#[derive(Clone, Debug, Default, Outgoing, Serialize)]
#[non_exhaustive]
#[incoming_derive(Clone, Serialize)]
pub struct UnsignedDeviceInfo<'a> {
    /// The display name which the user set on the device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_display_name: Option<&'a str>,
}

impl UnsignedDeviceInfo<'_> {
    /// Creates an empty `UnsignedDeviceInfo`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Checks whether all fields are empty / `None`.
    pub fn is_empty(&self) -> bool {
        self.device_display_name.is_none()
    }
}

impl IncomingUnsignedDeviceInfo {
    /// Checks whether all fields are empty / `None`.
    pub fn is_empty(&self) -> bool {
        self.device_display_name.is_none()
    }
}

impl CanBeEmpty for UnsignedDeviceInfo<'_> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl CanBeEmpty for IncomingUnsignedDeviceInfo {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}
