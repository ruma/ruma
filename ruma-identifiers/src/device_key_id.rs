//! Identifiers for device keys for end-to-end encryption.

use std::{num::NonZeroU8, str::FromStr};

use ruma_identifiers_validation::{key_algorithms::DeviceKeyAlgorithm, Error};

use crate::DeviceId;

/// A key algorithm and a device id, combined with a ':'
#[derive(Clone, Debug)]
pub struct DeviceKeyId {
    full_id: Box<str>,
    colon_idx: NonZeroU8,
}

impl DeviceKeyId {
    /// Returns key algorithm of the device key ID.
    pub fn algorithm(&self) -> DeviceKeyAlgorithm {
        DeviceKeyAlgorithm::from_str(&self.full_id[..self.colon_idx.get() as usize]).unwrap()
    }

    /// Returns device ID of the device key ID.
    pub fn device_id(&self) -> &DeviceId {
        (&self.full_id[self.colon_idx.get() as usize + 1..]).into()
    }
}

fn try_from<S>(key_id: S) -> Result<DeviceKeyId, Error>
where
    S: AsRef<str> + Into<Box<str>>,
{
    let colon_idx = ruma_identifiers_validation::device_key_id::validate(key_id.as_ref())?;
    Ok(DeviceKeyId { full_id: key_id.into(), colon_idx })
}

common_impls!(DeviceKeyId, try_from, "Device key ID with algorithm and device ID");

#[cfg(test)]
mod test {
    use std::convert::TryFrom;

    use ruma_identifiers_validation::{key_algorithms::DeviceKeyAlgorithm, Error};
    #[cfg(feature = "serde")]
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::DeviceKeyId;

    #[test]
    fn convert_device_key_id() {
        assert_eq!(
            DeviceKeyId::try_from("ed25519:JLAFKJWSCS")
                .expect("Failed to create device key ID.")
                .as_ref(),
            "ed25519:JLAFKJWSCS"
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_device_key_id() {
        let device_key_id = DeviceKeyId::try_from("ed25519:JLAFKJWSCS").unwrap();
        let serialized = to_json_value(device_key_id).unwrap();

        let expected = json!("ed25519:JLAFKJWSCS");
        assert_eq!(serialized, expected);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_device_key_id() {
        let deserialized: DeviceKeyId = from_json_value(json!("ed25519:JLAFKJWSCS")).unwrap();

        let expected = DeviceKeyId::try_from("ed25519:JLAFKJWSCS").unwrap();
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn missing_key_algorithm() {
        assert_eq!(DeviceKeyId::try_from(":JLAFKJWSCS").unwrap_err(), Error::UnknownKeyAlgorithm);
    }

    #[test]
    fn missing_delimiter() {
        assert_eq!(
            DeviceKeyId::try_from("ed25519|JLAFKJWSCS").unwrap_err(),
            Error::MissingDeviceKeyDelimiter,
        );
    }

    #[test]
    fn unknown_key_algorithm() {
        assert_eq!(
            DeviceKeyId::try_from("signed_curve25510:JLAFKJWSCS").unwrap_err(),
            Error::UnknownKeyAlgorithm,
        );
    }

    #[test]
    fn empty_device_id_ok() {
        assert!(DeviceKeyId::try_from("ed25519:").is_ok());
    }

    #[test]
    fn valid_key_algorithm() {
        let device_key_id = DeviceKeyId::try_from("ed25519:JLAFKJWSCS").unwrap();
        assert_eq!(device_key_id.algorithm(), DeviceKeyAlgorithm::Ed25519);
    }

    #[test]
    fn valid_device_id() {
        let device_key_id = DeviceKeyId::try_from("ed25519:JLAFKJWSCS").unwrap();
        assert_eq!(device_key_id.device_id(), "JLAFKJWSCS");
    }
}
