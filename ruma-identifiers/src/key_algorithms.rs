//! Key algorithms used in Matrix spec.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use strum::{AsRefStr, Display, EnumString};

/// The basic key algorithms in the specification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, AsRefStr, Display, EnumString)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub enum DeviceKeyAlgorithm {
    /// The Ed25519 signature algorithm.
    #[strum(to_string = "ed25519")]
    Ed25519,

    /// The Curve25519 ECDH algorithm.
    #[strum(to_string = "curve25519")]
    Curve25519,

    /// The Curve25519 ECDH algorithm, but the key also contains signatures
    #[strum(to_string = "signed_curve25519")]
    SignedCurve25519,
}

/// The server key algorithms defined in the Matrix spec.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, AsRefStr, Display, EnumString)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub enum ServerKeyAlgorithm {
    /// The Ed25519 signature algorithm.
    #[strum(to_string = "ed25519")]
    Ed25519,
}
