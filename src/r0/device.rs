//! Endpoints for managing devices.

use js_int::UInt;
use ruma_identifiers::DeviceId;
use serde::{Deserialize, Serialize};

pub mod delete_device;
pub mod delete_devices;
pub mod get_device;
pub mod get_devices;
pub mod update_device;

/// Information about a registered device.
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct Device {
    /// Device ID
    pub device_id: DeviceId,
    /// Public display name of the device.
    pub display_name: Option<String>,
    /// Most recently seen IP address of the session.
    pub ip: Option<String>,
    /// Unix timestamp that the session was last active.
    pub last_seen: Option<UInt>,
}
