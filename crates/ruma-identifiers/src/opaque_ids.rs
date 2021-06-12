//! Matrix device identifiers.

#[cfg(feature = "rand")]
use crate::generate_localpart;
use std::sync::Arc;
use std::rc::Rc;

opaque_identifier! {
    /// A Matrix key ID.
    ///
    /// Device identifiers in Matrix are completely opaque character sequences. This type is
    /// provided simply for its semantic value.
    pub type DeviceId;
}

impl DeviceId {
    /// Generates a random `DeviceId`, suitable for assignment to a new device.
    #[cfg(feature = "rand")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    pub fn new() -> Box<Self> {
        Self::from_owned(generate_localpart(8))
    }
}

impl<'a> From<&'a DeviceId> for Arc<DeviceId> {
    fn from(s: &DeviceId) -> Arc<DeviceId> {
        Arc::<DeviceId([str])>::from(s.0.as_bytes())
    }
}

impl<'a> From<&'a DeviceId> for Rc<DeviceId> {
    fn from(s: &DeviceId) -> Rc<DeviceId> {
        Rc::<DeviceId([str])>::from(s.0.as_bytes())
    }
}


opaque_identifier! {
    /// A Matrix key identifier.
    ///
    /// Key identifiers in Matrix are opaque character sequences of `[a-zA-Z_]`. This type is
    /// provided simply for its semantic value.
    pub type KeyName;
}

#[cfg(all(test, feature = "rand"))]
mod tests {
    use super::DeviceId;

    #[test]
    fn generate_device_id() {
        assert_eq!(DeviceId::new().as_str().len(), 8);
    }
}
