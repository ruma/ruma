#[cfg(feature = "rand")]
use crate::generate_localpart;

/// A Matrix key ID.
///
/// Device identifiers in Matrix are completely opaque character sequences. This type is provided
/// simply for its semantic value.
///
/// # Example
///
/// ```
/// use ruma_identifiers::{device_id, DeviceId};
///
/// let random_id = DeviceId::new();
/// assert_eq!(random_id.as_str().len(), 8);
///
/// let static_id = device_id!("01234567");
/// assert_eq!(static_id.as_str(), "01234567");
///
/// let ref_id: &DeviceId = "abcdefghi".into();
/// assert_eq!(ref_id.as_str(), "abcdefghi");
///
/// let owned_id: Box<DeviceId> = "ijklmnop".into();
/// assert_eq!(owned_id.as_str(), "ijklmnop");
/// ```
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceId(str);

opaque_identifier!(DeviceId);

impl DeviceId {
    /// Generates a random `DeviceId`, suitable for assignment to a new device.
    #[cfg(feature = "rand")]
    pub fn new() -> Box<Self> {
        Self::from_owned(generate_localpart(8))
    }
}

#[cfg(all(test, feature = "rand"))]
mod tests {
    use super::DeviceId;

    #[test]
    fn generate_device_id() {
        assert_eq!(DeviceId::new().as_str().len(), 8);
    }

    #[test]
    fn create_device_id_from_str() {
        let ref_id: &DeviceId = "abcdefgh".into();
        assert_eq!(ref_id.as_str(), "abcdefgh");
    }

    #[test]
    fn create_boxed_device_id_from_str() {
        let box_id: Box<DeviceId> = "12345678".into();
        assert_eq!(box_id.as_str(), "12345678");
    }

    #[test]
    fn create_device_id_from_box() {
        let box_str: Box<str> = "ijklmnop".into();
        let device_id: Box<DeviceId> = DeviceId::from_owned(box_str);
        assert_eq!(device_id.as_str(), "ijklmnop");
    }
}
