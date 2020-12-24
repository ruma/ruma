//! Key algorithms used in Matrix spec.

#[cfg(feature = "serde")]
use ruma_serde::{DeserializeFromCowStr, SerializeAsRefStr};
use ruma_serde_macros::{AsRefStr, DisplayAsRefStr, FromString};

/// The basic key algorithms in the specification.
///
/// This type can hold an arbitrary string. To check for algorithms that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, AsRefStr, DisplayAsRefStr, FromString)]
#[non_exhaustive]
#[ruma_enum(rename_all = "snake_case")]
#[cfg_attr(feature = "serde", derive(DeserializeFromCowStr, SerializeAsRefStr))]
pub enum DeviceKeyAlgorithm {
    /// The Ed25519 signature algorithm.
    Ed25519,

    /// The Curve25519 ECDH algorithm.
    Curve25519,

    /// The Curve25519 ECDH algorithm, but the key also contains signatures
    SignedCurve25519,

    #[doc(hidden)]
    _Custom(String),
}

/// The signing key algorithms defined in the Matrix spec.
///
/// This type can hold an arbitrary string. To check for algorithms that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, AsRefStr, DisplayAsRefStr, FromString)]
#[non_exhaustive]
#[ruma_enum(rename_all = "snake_case")]
#[cfg_attr(feature = "serde", derive(DeserializeFromCowStr, SerializeAsRefStr))]
pub enum SigningKeyAlgorithm {
    /// The Ed25519 signature algorithm.
    Ed25519,

    #[doc(hidden)]
    _Custom(String),
}

/// An encryption algorithm to be used to encrypt messages sent to a room.
///
/// This type can hold an arbitrary string. To check for algorithms that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, AsRefStr, DisplayAsRefStr, FromString)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(DeserializeFromCowStr, SerializeAsRefStr))]
pub enum EventEncryptionAlgorithm {
    /// Olm version 1 using Curve25519, AES-256, and SHA-256.
    #[ruma_enum(rename = "m.olm.v1.curve25519-aes-sha2")]
    OlmV1Curve25519AesSha2,

    /// Megolm version 1 using AES-256 and SHA-256.
    #[ruma_enum(rename = "m.megolm.v1.aes-sha2")]
    MegolmV1AesSha2,

    #[doc(hidden)]
    _Custom(String),
}

#[cfg(test)]
mod tests {
    use super::{DeviceKeyAlgorithm, SigningKeyAlgorithm};

    #[test]
    fn parse_device_key_algorithm() {
        assert_eq!(DeviceKeyAlgorithm::from("ed25519"), DeviceKeyAlgorithm::Ed25519);
        assert_eq!(DeviceKeyAlgorithm::from("curve25519"), DeviceKeyAlgorithm::Curve25519);
        assert_eq!(
            DeviceKeyAlgorithm::from("signed_curve25519"),
            DeviceKeyAlgorithm::SignedCurve25519
        );
    }

    #[test]
    fn parse_signing_key_algorithm() {
        assert_eq!(SigningKeyAlgorithm::from("ed25519"), SigningKeyAlgorithm::Ed25519);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn event_encryption_algorithm_serde() {
        use ruma_serde::test::serde_json_eq;
        use serde_json::json;

        use super::EventEncryptionAlgorithm;

        serde_json_eq(EventEncryptionAlgorithm::MegolmV1AesSha2, json!("m.megolm.v1.aes-sha2"));
        serde_json_eq(
            EventEncryptionAlgorithm::OlmV1Curve25519AesSha2,
            json!("m.olm.v1.curve25519-aes-sha2"),
        );
        serde_json_eq(
            EventEncryptionAlgorithm::_Custom("io.ruma.test".into()),
            json!("io.ruma.test"),
        );
    }
}
