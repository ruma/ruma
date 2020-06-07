//! Identifiers for device keys for end-to-end encryption.

use crate::{device_id::DeviceId, error::Error, key_algorithms::DeviceKeyAlgorithm};
use std::num::NonZeroU8;
use std::str::FromStr;

/// A key algorithm and a device id, combined with a ':'
#[derive(Clone, Debug)]
pub struct DeviceKeyId<T> {
    full_id: T,
    colon_idx: NonZeroU8,
}

impl<T> DeviceKeyId<T> {
    /// Returns key algorithm of the device key ID.
    pub fn algorithm(&self) -> DeviceKeyAlgorithm
    where
        T: AsRef<str>,
    {
        DeviceKeyAlgorithm::from_str(&self.full_id.as_ref()[..self.colon_idx.get() as usize])
            .unwrap()
    }

    /// Returns device ID of the device key ID.
    pub fn device_id(&self) -> DeviceId
    where
        T: AsRef<str>,
    {
        DeviceId::from(&self.full_id.as_ref()[self.colon_idx.get() as usize + 1..])
    }
}

fn try_from<S, T>(key_id: S) -> Result<DeviceKeyId<T>, Error>
where
    S: AsRef<str> + Into<T>,
{
    let key_str = key_id.as_ref();
    let colon_idx =
        NonZeroU8::new(key_str.find(':').ok_or(Error::MissingDeviceKeyDelimiter)? as u8)
            .ok_or(Error::UnknownKeyAlgorithm)?;

    DeviceKeyAlgorithm::from_str(&key_str[0..colon_idx.get() as usize])
        .map_err(|_| Error::UnknownKeyAlgorithm)?;

    Ok(DeviceKeyId {
        full_id: key_id.into(),
        colon_idx,
    })
}

common_impls!(
    DeviceKeyId,
    try_from,
    "Device key ID with algorithm and device ID"
);

#[cfg(test)]
mod test {
    use std::convert::TryFrom;

    #[cfg(feature = "serde")]
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::DeviceKeyId;
    use crate::{device_id::DeviceId, error::Error, key_algorithms::DeviceKeyAlgorithm};

    #[test]
    fn convert_device_key_id() {
        assert_eq!(
            DeviceKeyId::<&str>::try_from("ed25519:JLAFKJWSCS")
                .expect("Failed to create device key ID.")
                .as_ref(),
            "ed25519:JLAFKJWSCS"
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_device_key_id() {
        let device_key_id = DeviceKeyId::<&str>::try_from("ed25519:JLAFKJWSCS").unwrap();
        let serialized = to_json_value(device_key_id).unwrap();

        let expected = json!("ed25519:JLAFKJWSCS");
        assert_eq!(serialized, expected);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_device_key_id() {
        let deserialized: DeviceKeyId<_> = from_json_value(json!("ed25519:JLAFKJWSCS")).unwrap();

        let expected = DeviceKeyId::try_from("ed25519:JLAFKJWSCS").unwrap();
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn missing_key_algorithm() {
        assert_eq!(
            DeviceKeyId::<&str>::try_from(":JLAFKJWSCS").unwrap_err(),
            Error::UnknownKeyAlgorithm
        );
    }

    #[test]
    fn missing_delimiter() {
        assert_eq!(
            DeviceKeyId::<&str>::try_from("ed25519|JLAFKJWSCS").unwrap_err(),
            Error::MissingDeviceKeyDelimiter,
        );
    }

    #[test]
    fn unknown_key_algorithm() {
        assert_eq!(
            DeviceKeyId::<&str>::try_from("signed_curve25510:JLAFKJWSCS").unwrap_err(),
            Error::UnknownKeyAlgorithm,
        );
    }

    #[test]
    fn empty_device_id_ok() {
        assert!(DeviceKeyId::<&str>::try_from("ed25519:").is_ok());
    }

    #[test]
    fn valid_key_algorithm() {
        let device_key_id = DeviceKeyId::<&str>::try_from("ed25519:JLAFKJWSCS").unwrap();
        assert_eq!(device_key_id.algorithm(), DeviceKeyAlgorithm::Ed25519);
    }

    #[test]
    fn valid_device_id() {
        let device_key_id = DeviceKeyId::<&str>::try_from("ed25519:JLAFKJWSCS").unwrap();
        assert_eq!(device_key_id.device_id(), DeviceId::from("JLAFKJWSCS"));
    }
}
