//! Verification of digital signatures.

use ruma_common::SigningKeyAlgorithm;

use crate::{VerificationError, ed25519::Ed25519Verifier};

/// A digital signature verifier.
pub(crate) trait Verifier {
    /// The error type returned by the verifier.
    type Error: std::error::Error + Into<VerificationError>;

    /// Use a public key to verify a signature against the JSON object that was signed.
    ///
    /// # Parameters
    ///
    /// * `public_key`: The raw bytes of the public key of the key pair used to sign the message.
    /// * `signature`: The raw bytes of the signature to verify.
    /// * `message`: The raw bytes of the message that was signed.
    ///
    /// # Errors
    ///
    /// Returns an error if verification fails.
    fn verify_json(
        &self,
        public_key: &[u8],
        signature: &[u8],
        message: &[u8],
    ) -> Result<(), Self::Error>;
}

/// A value returned when an event is successfully verified.
///
/// Event verification involves verifying both signatures and a content hash. It is possible for
/// the signatures on an event to be valid, but for the hash to be different than the one
/// calculated during verification. This is not necessarily an error condition, as it may indicate
/// that the event has been redacted. In this case, receiving homeservers should store a redacted
/// version of the event.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
pub enum Verified {
    /// All signatures are valid and the content hashes match.
    All,

    /// All signatures are valid but the content hashes don't match.
    ///
    /// This may indicate a redacted event.
    Signatures,
}

/// Get the verifier for the given algorithm, if it is supported.
pub(crate) fn verifier_from_algorithm(
    algorithm: &SigningKeyAlgorithm,
) -> Option<impl Verifier + use<>> {
    match algorithm {
        SigningKeyAlgorithm::Ed25519 => Some(Ed25519Verifier),
        _ => None,
    }
}
