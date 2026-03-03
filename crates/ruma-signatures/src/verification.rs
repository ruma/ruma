//! Verification of digital signatures.

use ed25519_dalek::{
    Verifier as _, VerifyingKey as Ed25519VerifyingKey, ed25519::Signature as Ed25519Signature,
};
use ruma_common::SigningKeyAlgorithm;
use thiserror::Error;

use crate::VerificationError;

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

/// A verifier for Ed25519 digital signatures.
#[derive(Debug, Default)]
pub(crate) struct Ed25519Verifier;

impl Verifier for Ed25519Verifier {
    type Error = Ed25519VerificationError;

    fn verify_json(
        &self,
        public_key: &[u8],
        signature: &[u8],
        message: &[u8],
    ) -> Result<(), Self::Error> {
        Ed25519VerifyingKey::try_from(public_key)
            .map_err(Ed25519VerificationError::InvalidPublicKey)?
            .verify(
                message,
                &Ed25519Signature::from_bytes(&signature.try_into().map_err(|_| {
                    Ed25519VerificationError::InvalidSignatureLength {
                        expected: Ed25519Signature::BYTE_SIZE,
                        found: signature.len(),
                    }
                })?),
            )
            .map_err(Ed25519VerificationError::SignatureVerification)
    }
}

/// Errors relating to the verification of ed25519 signatures.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Ed25519VerificationError {
    /// The provided ed25519 public key is invalid.
    #[error("Invalid ed25519 public key: {0}")]
    InvalidPublicKey(#[source] ed25519_dalek::SignatureError),

    /// The provided signature has an invalid length.
    #[error("Invalid ed25519 signature length: expected {expected}, found {found}")]
    InvalidSignatureLength {
        /// The expected length of the signature.
        expected: usize,

        /// The actual length of the signature.
        found: usize,
    },

    /// The signature verification failed.
    #[error("ed25519 signature verification failed: {0}")]
    SignatureVerification(#[source] ed25519_dalek::SignatureError),
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
