//! Matrix device identifiers.

use crate::generate_localpart;

///  A Matrix device ID.
///
///  Device identifiers in Matrix are completely opaque character sequences. This type alias is
///  provided simply for its semantic value.
pub type DeviceId = String;

/// Generates a random `DeviceId`, suitable for assignment to a new device.
pub fn generate() -> DeviceId {
    generate_localpart(8)
}

#[cfg(test)]
mod tests {
    use super::generate;

    #[test]
    fn generate_device_id() {
        assert_eq!(generate().len(), 8);
    }
}
