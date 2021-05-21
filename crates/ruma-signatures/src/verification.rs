//! Verification of digital signatures.

use std::convert::TryInto;

use ed25519_dalek::{PublicKey, Verifier as _};

use crate::Error;

/// A digital signature verifier.
pub trait Verifier {
    /// Use a public key to verify a signature against the JSON object that was signed.
    ///
    /// # Parameters
    ///
    /// * public_key: The raw bytes of the public key of the key pair used to sign the message.
    /// * signature: The raw bytes of the signature to verify.
    /// * message: The raw bytes of the message that was signed.
    ///
    /// # Errors
    ///
    /// Returns an error if verification fails.
    fn verify_json(&self, public_key: &[u8], signature: &[u8], message: &[u8])
        -> Result<(), Error>;
}

/// A verifier for Ed25519 digital signatures.
#[derive(Debug, Default, PartialEq)]
pub struct Ed25519Verifier;

impl Verifier for Ed25519Verifier {
    fn verify_json(
        &self,
        public_key: &[u8],
        signature: &[u8],
        message: &[u8],
    ) -> Result<(), Error> {
        PublicKey::from_bytes(public_key)
            .map_err(|e| Error::new(format!("Could not parse public key: {:?}", e)))?
            .verify(
                message,
                &signature
                    .try_into()
                    .map_err(|e| Error::new(format!("Could not parse signature: {:?}", e)))?,
            )
            .map_err(|e| Error::new(format!("Could not verify signature: {:?}", e)))
    }
}

/// A value returned when an event is successfully verified.
///
/// Event verification involves verifying both signatures and a content hash. It is possible for
/// the signatures on an event to be valid, but for the hash to be different than the one
/// calculated during verification. This is not necessarily an error condition, as it may indicate
/// that the event has been redacted. In this case, receiving homeservers should store a redacted
/// version of the event.
#[derive(Clone, Debug, Hash, PartialEq)]
pub enum Verified {
    /// All signatures are valid and the content hashes match.
    All,

    /// All signatures are valid but the content hashes don't match.
    ///
    /// This may indicate a redacted event.
    Signatures,
}
