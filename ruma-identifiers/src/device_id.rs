//! Matrix device identifiers.

use std::{
    fmt::{self, Display},
    mem,
};

#[cfg(feature = "rand")]
use crate::generate_localpart;

/// A Matrix device ID.
///
/// Device identifiers in Matrix are completely opaque character sequences. This type alias is
/// provided simply for its semantic value.
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(transparent))]
pub struct DeviceId(str);

impl DeviceId {
    #[allow(clippy::transmute_ptr_to_ptr)]
    fn from_borrowed(s: &str) -> &Self {
        unsafe { mem::transmute(s) }
    }

    fn from_owned(s: Box<str>) -> Box<Self> {
        unsafe { mem::transmute(s) }
    }

    fn into_owned(self: Box<Self>) -> Box<str> {
        unsafe { mem::transmute(self) }
    }

    /// Generates a random `DeviceId`, suitable for assignment to a new device.
    #[cfg(feature = "rand")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    pub fn new() -> Box<Self> {
        Self::from_owned(generate_localpart(8))
    }

    /// Creates a string slice from this `DeviceId`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Clone for Box<DeviceId> {
    fn clone(&self) -> Self {
        (**self).to_owned()
    }
}

impl ToOwned for DeviceId {
    type Owned = Box<DeviceId>;

    fn to_owned(&self) -> Self::Owned {
        Self::from_owned(self.0.to_owned().into_boxed_str())
    }
}

impl AsRef<str> for DeviceId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for Box<DeviceId> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> From<&'a str> for &'a DeviceId {
    fn from(s: &'a str) -> Self {
        DeviceId::from_borrowed(s)
    }
}

impl From<&str> for Box<DeviceId> {
    fn from(s: &str) -> Self {
        DeviceId::from_owned(s.into())
    }
}

impl From<String> for Box<DeviceId> {
    fn from(s: String) -> Self {
        DeviceId::from_owned(s.into())
    }
}

impl From<Box<DeviceId>> for String {
    fn from(id: Box<DeviceId>) -> Self {
        id.into_owned().into()
    }
}

impl Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Box<DeviceId> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        crate::deserialize_id(deserializer, "An IP address or hostname")
    }
}

partial_eq_string!(DeviceId);
partial_eq_string!(Box<DeviceId>);

/// Generates a random `DeviceId`, suitable for assignment to a new device.
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
#[deprecated = "use DeviceId::new instead"]
pub fn generate() -> Box<DeviceId> {
    DeviceId::new()
}

#[cfg(all(test, feature = "rand"))]
mod tests {
    use super::DeviceId;

    #[test]
    fn generate_device_id() {
        assert_eq!(DeviceId::new().as_str().len(), 8);
    }
}
