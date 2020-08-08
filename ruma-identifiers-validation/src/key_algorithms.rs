//! Key algorithms used in Matrix spec.

use std::convert::TryFrom;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use strum::{AsRefStr, Display, EnumString};

/// The basic key algorithms in the specification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, AsRefStr, Display, EnumString)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[non_exhaustive]
#[strum(serialize_all = "snake_case")]
pub enum DeviceKeyAlgorithm {
    /// The Ed25519 signature algorithm.
    Ed25519,

    /// The Curve25519 ECDH algorithm.
    Curve25519,

    /// The Curve25519 ECDH algorithm, but the key also contains signatures
    SignedCurve25519,
}

impl TryFrom<&'_ str> for DeviceKeyAlgorithm {
    type Error = strum::ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for DeviceKeyAlgorithm {
    type Error = strum::ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

/// The server key algorithms defined in the Matrix spec.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, AsRefStr, Display, EnumString)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[non_exhaustive]
#[strum(serialize_all = "snake_case")]
pub enum ServerKeyAlgorithm {
    /// The Ed25519 signature algorithm.
    Ed25519,
}

impl TryFrom<&'_ str> for ServerKeyAlgorithm {
    type Error = strum::ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for ServerKeyAlgorithm {
    type Error = strum::ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::{DeviceKeyAlgorithm, ServerKeyAlgorithm};

    #[test]
    fn parse_device_key_algorithm() {
        assert_eq!("ed25519".parse(), Ok(DeviceKeyAlgorithm::Ed25519));
        assert_eq!("curve25519".parse(), Ok(DeviceKeyAlgorithm::Curve25519));
        assert_eq!("signed_curve25519".parse(), Ok(DeviceKeyAlgorithm::SignedCurve25519));
    }

    #[test]
    fn parse_server_key_algorithm() {
        assert_eq!("ed25519".parse(), Ok(ServerKeyAlgorithm::Ed25519));
    }
}
