//! Endpoints for managing dehydrated devices.

use ruma_common::serde::StringEnum;
use serde::{Deserialize, Serialize};

use crate::PrivOwnedStr;

pub mod delete_dehydrated_device;
pub mod get_dehydrated_device;
pub mod get_events;
pub mod put_dehydrated_device;

/// Data for a dehydrated device.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "Helper", into = "Helper")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum DehydratedDeviceData {
    /// The `org.matrix.msc3814.v1.olm` variant of a dehydrated device.
    V1(DehydratedDeviceV1),
}

impl DehydratedDeviceData {
    /// Get the algorithm this dehydrated device uses.
    pub fn algorithm(&self) -> DeviceDehydrationAlgorithm {
        match self {
            DehydratedDeviceData::V1(_) => DeviceDehydrationAlgorithm::V1,
        }
    }
}

/// The `org.matrix.msc3814.v1.olm` variant of a dehydrated device.
#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct DehydratedDeviceV1 {
    /// The pickle of the `Olm` account of the device.
    ///
    /// The pickle will contain the private parts of the long-term identity keys of the device as
    /// well as a collection of one-time keys.
    pub device_pickle: String,
}

impl DehydratedDeviceV1 {
    /// Create a [`DehydratedDeviceV1`] struct from a device pickle.
    pub fn new(device_pickle: String) -> Self {
        Self { device_pickle }
    }
}

/// The algorithms used for dehydrated devices.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, StringEnum)]
#[non_exhaustive]
pub enum DeviceDehydrationAlgorithm {
    /// The `org.matrix.msc3814.v1.olm` device dehydration algorithm.
    #[ruma_enum(rename = "org.matrix.msc3814.v1.olm")]
    V1,
    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

#[derive(Deserialize, Serialize)]
struct Helper {
    algorithm: DeviceDehydrationAlgorithm,
    device_pickle: String,
}

impl TryFrom<Helper> for DehydratedDeviceData {
    type Error = serde_json::Error;

    fn try_from(value: Helper) -> Result<Self, Self::Error> {
        match value.algorithm {
            DeviceDehydrationAlgorithm::V1 => Ok(DehydratedDeviceData::V1(DehydratedDeviceV1 {
                device_pickle: value.device_pickle,
            })),
            _ => Err(serde::de::Error::custom("Unsupported device dehydration algorithm.")),
        }
    }
}

impl From<DehydratedDeviceData> for Helper {
    fn from(value: DehydratedDeviceData) -> Self {
        let algorithm = value.algorithm();

        match value {
            DehydratedDeviceData::V1(d) => Self { algorithm, device_pickle: d.device_pickle },
        }
    }
}
