//! Key algorithms used in Matrix spec.

use ruma_macros::StringEnum;

use crate::PrivOwnedStr;

/// The algorithms for the [device keys] defined in the Matrix spec.
///
/// [device keys]: https://spec.matrix.org/latest/client-server-api/#device-keys
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, StringEnum)]
#[non_exhaustive]
#[ruma_enum(rename_all = "snake_case")]
pub enum DeviceKeyAlgorithm {
    /// The Ed25519 signature algorithm.
    Ed25519,

    /// The Curve25519 ECDH algorithm.
    Curve25519,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The signing key algorithms defined in the Matrix spec.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, StringEnum)]
#[non_exhaustive]
#[ruma_enum(rename_all = "snake_case")]
pub enum SigningKeyAlgorithm {
    /// The Ed25519 signature algorithm.
    Ed25519,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// An encryption algorithm to be used to encrypt messages sent to a room.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, StringEnum)]
#[non_exhaustive]
pub enum EventEncryptionAlgorithm {
    /// Olm version 1 using Curve25519, AES-256, and SHA-256.
    #[ruma_enum(rename = "m.olm.v1.curve25519-aes-sha2")]
    OlmV1Curve25519AesSha2,

    /// Megolm version 1 using AES-256 and SHA-256.
    #[ruma_enum(rename = "m.megolm.v1.aes-sha2")]
    MegolmV1AesSha2,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// A key algorithm to be used to generate a key from a passphrase.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, StringEnum)]
#[non_exhaustive]
pub enum KeyDerivationAlgorithm {
    /// PBKDF2
    #[ruma_enum(rename = "m.pbkdf2")]
    Pbkfd2,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The algorithms for [one-time and fallback keys] defined in the Matrix spec.
///
/// [one-time and fallback keys]: https://spec.matrix.org/latest/client-server-api/#one-time-and-fallback-keys
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, StringEnum)]
#[non_exhaustive]
#[ruma_enum(rename_all = "snake_case")]
pub enum OneTimeKeyAlgorithm {
    /// The Curve25519 ECDH algorithm, but the key also contains signatures.
    SignedCurve25519,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

#[cfg(test)]
mod tests {
    use super::{DeviceKeyAlgorithm, OneTimeKeyAlgorithm, SigningKeyAlgorithm};

    #[test]
    fn parse_device_key_algorithm() {
        assert_eq!(DeviceKeyAlgorithm::from("ed25519"), DeviceKeyAlgorithm::Ed25519);
        assert_eq!(DeviceKeyAlgorithm::from("curve25519"), DeviceKeyAlgorithm::Curve25519);
    }

    #[test]
    fn parse_signing_key_algorithm() {
        assert_eq!(SigningKeyAlgorithm::from("ed25519"), SigningKeyAlgorithm::Ed25519);
    }

    #[test]
    fn event_encryption_algorithm_serde() {
        use serde_json::json;

        use super::EventEncryptionAlgorithm;
        use crate::serde::test::serde_json_eq;

        serde_json_eq(EventEncryptionAlgorithm::MegolmV1AesSha2, json!("m.megolm.v1.aes-sha2"));
        serde_json_eq(
            EventEncryptionAlgorithm::OlmV1Curve25519AesSha2,
            json!("m.olm.v1.curve25519-aes-sha2"),
        );
        serde_json_eq(EventEncryptionAlgorithm::from("io.ruma.test"), json!("io.ruma.test"));
    }

    #[test]
    fn key_derivation_algorithm_serde() {
        use serde_json::json;

        use super::KeyDerivationAlgorithm;
        use crate::serde::test::serde_json_eq;

        serde_json_eq(KeyDerivationAlgorithm::Pbkfd2, json!("m.pbkdf2"));
    }

    #[test]
    fn parse_one_time_key_algorithm() {
        assert_eq!(
            OneTimeKeyAlgorithm::from("signed_curve25519"),
            OneTimeKeyAlgorithm::SignedCurve25519
        );
    }
}
