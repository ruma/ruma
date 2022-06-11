//! Identifiers for device keys for end-to-end encryption.

use ruma_macros::IdZst;

use super::{crypto_algorithms::DeviceKeyAlgorithm, DeviceId};

/// A key algorithm and a device id, combined with a ':'.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdZst)]
#[ruma_id(validate = ruma_identifiers_validation::device_key_id::validate)]
pub struct DeviceKeyId(str);

impl DeviceKeyId {
    /// Create a `DeviceKeyId` from a `DeviceKeyAlgorithm` and a `DeviceId`.
    pub fn from_parts(algorithm: DeviceKeyAlgorithm, device_id: &DeviceId) -> OwnedDeviceKeyId {
        let algorithm: &str = algorithm.as_ref();
        let device_id: &str = device_id.as_ref();

        let mut res = String::with_capacity(algorithm.len() + 1 + device_id.len());
        res.push_str(algorithm);
        res.push(':');
        res.push_str(device_id);

        Self::from_borrowed(&res).to_owned()
    }

    /// Returns key algorithm of the device key ID.
    pub fn algorithm(&self) -> DeviceKeyAlgorithm {
        self.as_str()[..self.colon_idx()].into()
    }

    /// Returns device ID of the device key ID.
    pub fn device_id(&self) -> &DeviceId {
        self.as_str()[self.colon_idx() + 1..].into()
    }

    fn colon_idx(&self) -> usize {
        self.as_str().find(':').unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::{DeviceKeyId, OwnedDeviceKeyId};
    use crate::identifiers::{crypto_algorithms::DeviceKeyAlgorithm, IdParseError};

    #[test]
    fn convert_device_key_id() {
        assert_eq!(
            <&DeviceKeyId>::try_from("ed25519:JLAFKJWSCS")
                .expect("Failed to create device key ID."),
            "ed25519:JLAFKJWSCS"
        );
    }

    #[test]
    fn serialize_device_key_id() {
        let device_key_id = <&DeviceKeyId>::try_from("ed25519:JLAFKJWSCS").unwrap();
        let serialized = serde_json::to_value(device_key_id).unwrap();

        assert_eq!(serialized, serde_json::json!("ed25519:JLAFKJWSCS"));
    }

    #[test]
    fn deserialize_device_key_id() {
        let deserialized: OwnedDeviceKeyId =
            serde_json::from_value(serde_json::json!("ed25519:JLAFKJWSCS")).unwrap();

        let expected = <&DeviceKeyId>::try_from("ed25519:JLAFKJWSCS").unwrap();
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn missing_key_algorithm() {
        assert_eq!(<&DeviceKeyId>::try_from(":JLAFKJWSCS").unwrap_err(), IdParseError::Empty);
    }

    #[test]
    fn missing_delimiter() {
        assert_eq!(
            <&DeviceKeyId>::try_from("ed25519|JLAFKJWSCS").unwrap_err(),
            IdParseError::MissingColon,
        );
    }

    #[test]
    fn empty_device_id_ok() {
        <&DeviceKeyId>::try_from("ed25519:").unwrap();
    }

    #[test]
    fn valid_key_algorithm() {
        let device_key_id = <&DeviceKeyId>::try_from("ed25519:JLAFKJWSCS").unwrap();
        assert_eq!(device_key_id.algorithm(), DeviceKeyAlgorithm::Ed25519);
    }

    #[test]
    fn valid_device_id() {
        let device_key_id = <&DeviceKeyId>::try_from("ed25519:JLAFKJWSCS").unwrap();
        assert_eq!(device_key_id.device_id(), "JLAFKJWSCS");
    }
}
