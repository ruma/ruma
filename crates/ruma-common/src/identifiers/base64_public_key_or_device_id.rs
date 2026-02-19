use ruma_macros::IdDst;

use super::{
    Base64PublicKey, DeviceId, IdParseError, KeyName, OwnedBase64PublicKey, OwnedDeviceId,
};

/// A Matrix ID that can be either a [`DeviceId`] or a [`Base64PublicKey`].
///
/// Device identifiers in Matrix are completely opaque character sequences and cross-signing keys
/// are identified by their base64-encoded public key. This type is provided simply for its semantic
/// value.
///
/// It is not recommended to construct this type directly, it should instead be converted from a
/// [`DeviceId`] or a [`Base64PublicKey`].
///
/// # Example
///
/// ```
/// use ruma_common::{Base64PublicKeyOrDeviceId, OwnedBase64PublicKeyOrDeviceId};
///
/// let ref_id: &Base64PublicKeyOrDeviceId = "abcdefghi".into();
/// assert_eq!(ref_id.as_str(), "abcdefghi");
///
/// let owned_id: OwnedBase64PublicKeyOrDeviceId = "ijklmnop".into();
/// assert_eq!(owned_id.as_str(), "ijklmnop");
/// ```
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdDst)]
pub struct Base64PublicKeyOrDeviceId(str);

impl KeyName for Base64PublicKeyOrDeviceId {
    fn validate(_s: &str) -> Result<(), IdParseError> {
        Ok(())
    }
}

impl KeyName for OwnedBase64PublicKeyOrDeviceId {
    fn validate(_s: &str) -> Result<(), IdParseError> {
        Ok(())
    }
}

impl<'a> From<&'a DeviceId> for &'a Base64PublicKeyOrDeviceId {
    fn from(value: &'a DeviceId) -> Self {
        Self::from(value.as_str())
    }
}

impl From<OwnedDeviceId> for OwnedBase64PublicKeyOrDeviceId {
    fn from(value: OwnedDeviceId) -> Self {
        unsafe { Self::from_raw(value.into_raw() as *const Base64PublicKeyOrDeviceId) }
    }
}

impl<'a> From<&'a Base64PublicKey> for &'a Base64PublicKeyOrDeviceId {
    fn from(value: &'a Base64PublicKey) -> Self {
        Self::from(value.as_str())
    }
}

impl From<OwnedBase64PublicKey> for OwnedBase64PublicKeyOrDeviceId {
    fn from(value: OwnedBase64PublicKey) -> Self {
        unsafe { Self::from_raw(value.into_raw() as *const Base64PublicKeyOrDeviceId) }
    }
}

#[cfg(test)]
mod tests {
    use super::OwnedBase64PublicKeyOrDeviceId;
    use crate::{Base64PublicKey, OwnedBase64PublicKey, OwnedDeviceId};

    #[test]
    fn convert_owned_device_id_to_owned_base64_public_key_or_device_id() {
        let device_id: OwnedDeviceId = "MYDEVICE".into();
        let mixed: OwnedBase64PublicKeyOrDeviceId = device_id.into();

        assert_eq!(mixed.as_str(), "MYDEVICE");
    }

    #[test]
    fn convert_owned_base64_public_key_to_owned_base64_public_key_or_device_id() {
        let base64_public_key: OwnedBase64PublicKey =
            <&Base64PublicKey>::try_from("base64+master+public+key").unwrap().to_owned();
        let mixed: OwnedBase64PublicKeyOrDeviceId = base64_public_key.into();

        assert_eq!(mixed.as_str(), "base64+master+public+key");
    }
}
