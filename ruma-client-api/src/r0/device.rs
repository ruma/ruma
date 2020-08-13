//! Endpoints for managing devices.

use std::time::SystemTime;

use ruma_identifiers::DeviceIdBox;
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
    pub device_id: DeviceIdBox,

    /// Public display name of the device.
    pub display_name: Option<String>,

    /// Most recently seen IP address of the session.
    pub last_seen_ip: Option<String>,

    /// Unix timestamp that the session was last active.
    #[serde(
        with = "ruma_serde::time::opt_ms_since_unix_epoch",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub last_seen_ts: Option<SystemTime>,
}
