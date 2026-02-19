use ruma_macros::ruma_id;

#[cfg(feature = "rand")]
use super::generate_localpart;
use super::{IdParseError, KeyName};

/// A Matrix device ID.
///
/// Device identifiers in Matrix are completely opaque character sequences. This type is provided
/// simply for its semantic value.
///
/// # Example
///
/// ```
/// use ruma_common::{DeviceId, device_id};
///
/// # #[cfg(feature = "rand")] {
/// let random_id = DeviceId::new();
/// assert_eq!(random_id.as_str().len(), 10);
/// # }
///
/// let macro_id = device_id!("01234567");
/// assert_eq!(macro_id, "01234567");
///
/// let id: DeviceId = "abcdefghi".into();
/// assert_eq!(id, "abcdefghi");
/// ```
#[ruma_id]
pub struct DeviceId;

impl DeviceId {
    /// Generates a random `DeviceId`, suitable for assignment to a new device.
    #[cfg(feature = "rand")]
    pub fn new() -> Self {
        Self::from_box_str_unchecked(generate_localpart(10))
    }
}

impl KeyName for DeviceId {
    fn validate(_s: &str) -> Result<(), IdParseError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::DeviceId;

    #[cfg(feature = "rand")]
    #[test]
    fn generate_device_id() {
        assert_eq!(DeviceId::new().as_str().len(), 10);
    }

    #[test]
    fn create_device_id_from_str() {
        let ref_id = DeviceId::from("abcdefgh");
        assert_eq!(ref_id.as_str(), "abcdefgh");
    }

    #[test]
    fn create_device_id_from_box() {
        let box_str = Box::<str>::from("ijklmnop");
        assert_eq!(DeviceId::from(box_str).as_str(), "ijklmnop");
    }
}
