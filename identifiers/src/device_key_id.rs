//! Identifiers for device keys for end-to-end encryption.

use std::{convert::TryInto, fmt, num::NonZeroU8};

use crate::{crypto_algorithms::DeviceKeyAlgorithm, DeviceId, Error};

/// A key algorithm and a device id, combined with a ':'.
#[derive(Clone)]
pub struct DeviceKeyId {
    full_id: Box<str>,
    colon_idx: NonZeroU8,
}

impl fmt::Debug for DeviceKeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.full_id)
    }
}

impl DeviceKeyId {
    /// Create a `DeviceKeyId` from a `DeviceKeyAlgorithm` and a `DeviceId`.
    pub fn from_parts(algorithm: DeviceKeyAlgorithm, device_id: &DeviceId) -> Self {
        let algorithm: &str = algorithm.as_ref();
        let device_id: &str = device_id.as_ref();

        let mut res = String::with_capacity(algorithm.len() + 1 + device_id.len());
        res.push_str(algorithm);
        res.push(':');
        res.push_str(device_id);

        let colon_idx =
            NonZeroU8::new(algorithm.len().try_into().expect("no algorithm name len > 255"))
                .expect("no empty algorithm name");

        DeviceKeyId { full_id: res.into(), colon_idx }
    }

    /// Returns key algorithm of the device key ID.
    pub fn algorithm(&self) -> DeviceKeyAlgorithm {
        self.full_id[..self.colon_idx.get() as usize].into()
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
mod tests {
    use std::convert::TryFrom;

    use super::DeviceKeyId;
    use crate::{crypto_algorithms::DeviceKeyAlgorithm, Error};

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
        let serialized = serde_json::to_value(device_key_id).unwrap();

        assert_eq!(serialized, serde_json::json!("ed25519:JLAFKJWSCS"));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_device_key_id() {
        let deserialized: DeviceKeyId =
            serde_json::from_value(serde_json::json!("ed25519:JLAFKJWSCS")).unwrap();

        let expected = DeviceKeyId::try_from("ed25519:JLAFKJWSCS").unwrap();
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn missing_key_algorithm() {
        assert_eq!(DeviceKeyId::try_from(":JLAFKJWSCS").unwrap_err(), Error::InvalidKeyAlgorithm);
    }

    #[test]
    fn missing_delimiter() {
        assert_eq!(
            DeviceKeyId::try_from("ed25519|JLAFKJWSCS").unwrap_err(),
            Error::MissingDelimiter,
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
