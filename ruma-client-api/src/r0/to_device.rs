//! Endpoints for client devices to exchange information not persisted in room DAG.

use std::{
    convert::TryFrom,
    fmt::{Display, Formatter, Result as FmtResult},
};

use ruma_identifiers::DeviceIdBox;
use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer, Serialize, Serializer,
};

pub mod send_event_to_device;

/// Represents one or all of a user's devices.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeviceIdOrAllDevices {
    /// Represents a device Id for one of a user's devices.
    DeviceId(DeviceIdBox),

    /// Represents all devices for a user.
    AllDevices,
}

impl Display for DeviceIdOrAllDevices {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            DeviceIdOrAllDevices::DeviceId(device_id) => write!(f, "{}", device_id.to_string()),
            DeviceIdOrAllDevices::AllDevices => write!(f, "*"),
        }
    }
}

impl TryFrom<&str> for DeviceIdOrAllDevices {
    type Error = &'static str;
    fn try_from(device_id_or_all_devices: &str) -> Result<Self, Self::Error> {
        if device_id_or_all_devices.is_empty() {
            Err("Device identifier cannot be empty")
        } else if "*" == device_id_or_all_devices {
            Ok(DeviceIdOrAllDevices::AllDevices)
        } else {
            Ok(DeviceIdOrAllDevices::DeviceId(device_id_or_all_devices.into()))
        }
    }
}

impl Serialize for DeviceIdOrAllDevices {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::DeviceId(device_id) => device_id.serialize(serializer),
            Self::AllDevices => serializer.serialize_str("*"),
        }
    }
}

impl<'de> Deserialize<'de> for DeviceIdOrAllDevices {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = ruma_serde::deserialize_cow_str(deserializer)?;
        DeviceIdOrAllDevices::try_from(s.as_ref()).map_err(|_| {
            de::Error::invalid_value(Unexpected::Str(&s), &"a valid device identifier or '*'")
        })
    }
}
