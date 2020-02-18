//! Endpoints for client devices to exchange information not persisted in room DAG.

use std::{
    convert::TryFrom,
    fmt::{Display, Formatter, Result as FmtResult},
};

use ruma_identifiers::DeviceId;
use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer, Serialize, Serializer,
};

pub mod send_event_to_device;
/// Represents one or all of a user's devices.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum DeviceIdOrAllDevices {
    /// Represents a device Id for one of a user's devices.
    DeviceId(DeviceId),
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
            Ok(DeviceIdOrAllDevices::DeviceId(
                device_id_or_all_devices.to_string(),
            ))
        }
    }
}

impl Serialize for DeviceIdOrAllDevices {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::DeviceId(ref device_id) => serializer.serialize_str(&device_id),
            Self::AllDevices => serializer.serialize_str("*"),
        }
    }
}

impl<'de> Deserialize<'de> for DeviceIdOrAllDevices {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = &String::deserialize(deserializer)?;

        DeviceIdOrAllDevices::try_from(&value[..]).map_err(|_| {
            de::Error::invalid_value(Unexpected::Str(&value), &"a valid device identifier or '*'")
        })
    }
}
