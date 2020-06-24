//! Identifiers for device keys for end-to-end encryption.

use std::{mem, num::NonZeroU8, str::FromStr};

use slice_dst::StrWithHeader;

use crate::{
    error::Error, key_algorithms::DeviceKeyAlgorithm, util::CommonIdentHeader, DeviceIdRef,
};

/// A key algorithm and a device id, combined with a ':'
#[derive(Debug)]
#[repr(transparent)]
pub struct DeviceKeyId(StrWithHeader<CommonIdentHeader>);

// TODO: Add inherent method that creates Box<DeviceKeyId>

impl DeviceKeyId {
    /// Returns key algorithm of the device key ID.
    pub fn algorithm(&self) -> DeviceKeyAlgorithm {
        DeviceKeyAlgorithm::from_str(&self.0.str[..self.0.header.colon_idx.get() as usize]).unwrap()
    }

    /// Returns device ID of the device key ID.
    pub fn device_id(&self) -> DeviceIdRef<'_> {
        &self.0.str[self.0.header.colon_idx.get() as usize + 1..]
    }
}

impl Clone for Box<DeviceKeyId> {
    fn clone(&self) -> Self {
        new(self.0.header, &self.0.str)
    }
}

fn new(header: CommonIdentHeader, s: &str) -> Box<DeviceKeyId> {
    let inner: Box<_> = StrWithHeader::new(header, s);
    unsafe { mem::transmute(inner) }
}

fn try_from(key_id: &str) -> Result<Box<DeviceKeyId>, Error> {
    let colon_idx = NonZeroU8::new(key_id.find(':').ok_or(Error::MissingDeviceKeyDelimiter)? as u8)
        .ok_or(Error::UnknownKeyAlgorithm)?;

    DeviceKeyAlgorithm::from_str(&key_id[0..colon_idx.get() as usize])
        .map_err(|_| Error::UnknownKeyAlgorithm)?;

    Ok(new(CommonIdentHeader { colon_idx }, key_id))
}

//common_impls!(DeviceKeyId, try_from, "Device key ID with algorithm and device ID");

impl ::std::convert::AsRef<str> for DeviceKeyId {
    fn as_ref(&self) -> &str {
        &self.0.str
    }
}

impl ::std::convert::TryFrom<&str> for Box<DeviceKeyId> {
    type Error = crate::error::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        try_from(s)
    }
}

impl ::std::convert::TryFrom<Box<str>> for Box<DeviceKeyId> {
    type Error = crate::error::Error;

    fn try_from(s: Box<str>) -> Result<Self, Self::Error> {
        try_from(&s)
    }
}

impl ::std::convert::TryFrom<String> for Box<DeviceKeyId> {
    type Error = crate::error::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        try_from(&s)
    }
}

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
            Box::<DeviceKeyId>::try_from("ed25519:JLAFKJWSCS")
                .expect("Failed to create device key ID.")
                .as_ref(),
            "ed25519:JLAFKJWSCS"
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_device_key_id() {
        let device_key_id = <&DeviceKeyId>::try_from("ed25519:JLAFKJWSCS").unwrap();
        let serialized = to_json_value(device_key_id).unwrap();

        let expected = json!("ed25519:JLAFKJWSCS");
        assert_eq!(serialized, expected);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_device_key_id() {
        let deserialized: Box<DeviceKeyId> = from_json_value(json!("ed25519:JLAFKJWSCS")).unwrap();

        let expected = <&DeviceKeyId>::try_from("ed25519:JLAFKJWSCS").unwrap();
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn missing_key_algorithm() {
        assert_eq!(
            <&DeviceKeyId>::try_from(":JLAFKJWSCS").unwrap_err(),
            Error::UnknownKeyAlgorithm
        );
    }

    #[test]
    fn missing_delimiter() {
        assert_eq!(
            <&DeviceKeyId>::try_from("ed25519|JLAFKJWSCS").unwrap_err(),
            Error::MissingDeviceKeyDelimiter,
        );
    }

    #[test]
    fn unknown_key_algorithm() {
        assert_eq!(
            <&DeviceKeyId>::try_from("signed_curve25510:JLAFKJWSCS").unwrap_err(),
            Error::UnknownKeyAlgorithm,
        );
    }

    #[test]
    fn empty_device_id_ok() {
        assert!(<&DeviceKeyId>::try_from("ed25519:").is_ok());
    }

    #[test]
    fn valid_key_algorithm() {
        let device_key_id = <&DeviceKeyId>::try_from("ed25519:JLAFKJWSCS").unwrap();
        assert_eq!(device_key_id.algorithm(), DeviceKeyAlgorithm::Ed25519);
    }

    #[test]
    fn valid_device_id() {
        let device_key_id = <&DeviceKeyId>::try_from("ed25519:JLAFKJWSCS").unwrap();
        assert_eq!(device_key_id.device_id(), DeviceId::from("JLAFKJWSCS"));
    }
}
