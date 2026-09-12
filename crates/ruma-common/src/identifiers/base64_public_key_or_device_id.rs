use ruma_macros::ruma_id;

use super::{Base64PublicKey, DeviceId, IdParseError, KeyName};

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
/// use ruma_common::Base64PublicKeyOrDeviceId;
///
/// let id: Base64PublicKeyOrDeviceId = "abcdefghi".into();
/// assert_eq!(id, "abcdefghi");
/// ```
///
/// [`DeviceId`]: super::DeviceId
#[ruma_id]
pub struct Base64PublicKeyOrDeviceId;

impl KeyName for Base64PublicKeyOrDeviceId {
    fn validate(_s: &str) -> Result<(), IdParseError> {
        Ok(())
    }
}

impl From<DeviceId> for Base64PublicKeyOrDeviceId {
    fn from(value: DeviceId) -> Self {
        unsafe { Self::from_inner_unchecked(value.into_inner()) }
    }
}

impl From<Base64PublicKey> for Base64PublicKeyOrDeviceId {
    fn from(value: Base64PublicKey) -> Self {
        unsafe { Self::from_inner_unchecked(value.into_inner()) }
    }
}

#[cfg(test)]
mod tests {
    use super::Base64PublicKeyOrDeviceId;
    use crate::{Base64PublicKey, DeviceId};

    #[test]
    fn convert_owned_device_id_to_base64_public_key_or_device_id() {
        let device_id: DeviceId = "MYDEVICE".into();
        let mixed: Base64PublicKeyOrDeviceId = device_id.into();

        assert_eq!(mixed, "MYDEVICE");
    }

    #[test]
    fn convert_base64_public_key_to_base64_public_key_or_device_id() {
        let base64_public_key: Base64PublicKey = "base64+master+public+key".try_into().unwrap();
        let mixed: Base64PublicKeyOrDeviceId = base64_public_key.into();

        assert_eq!(mixed, "base64+master+public+key");
    }
}
