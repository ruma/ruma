//! Matrix device identifiers.

#[cfg(feature = "rand")]
use crate::generate_localpart;

/// A Matrix device ID.
///
/// Device identifiers in Matrix are completely opaque character sequences. This type alias is
/// provided simply for its semantic value.
#[cfg(feature = "alloc")]
pub type DeviceId = alloc::string::String;

/// Generates a random `DeviceId`, suitable for assignment to a new device.
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub fn generate() -> DeviceId {
    generate_localpart(8)
}

#[cfg(all(test, feature = "rand"))]
mod tests {
    use super::generate;

    #[test]
    fn generate_device_id() {
        assert_eq!(generate().len(), 8);
    }
}
