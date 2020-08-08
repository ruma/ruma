//! Key algorithms used in Matrix spec.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use strum::{AsRefStr, Display, EnumString};

/// The basic key algorithms in the specification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, AsRefStr, Display, EnumString)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize), serde(rename_all = "snake_case"))]
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

/// The server key algorithms defined in the Matrix spec.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, AsRefStr, Display, EnumString)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize), serde(rename_all = "snake_case"))]
#[non_exhaustive]
#[strum(serialize_all = "snake_case")]
pub enum ServerKeyAlgorithm {
    /// The Ed25519 signature algorithm.
    Ed25519,
}
