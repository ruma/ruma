//! Endpoints for managing devices.

use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_identifiers::DeviceId;
use serde::{Deserialize, Serialize};

pub mod delete_device;
pub mod delete_devices;
pub mod get_device;
pub mod get_devices;
pub mod update_device;

/// Information about a registered device.
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Device {
    /// Device ID
    pub device_id: Box<DeviceId>,

    /// Public display name of the device.
    pub display_name: Option<String>,

    /// Most recently seen IP address of the session.
    pub last_seen_ip: Option<String>,

    /// Unix timestamp that the session was last active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen_ts: Option<MilliSecondsSinceUnixEpoch>,
}

impl Device {
    /// Creates a new `Device` with the given device ID.
    pub fn new(device_id: Box<DeviceId>) -> Self {
        Self { device_id, display_name: None, last_seen_ip: None, last_seen_ts: None }
    }
}
