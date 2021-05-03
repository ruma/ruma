//! Public and private key pairs.

use std::{
    collections::BTreeMap,
    fmt::{Debug, Formatter, Result as FmtResult},
};

use ring::signature::{Ed25519KeyPair as RingEd25519KeyPair, KeyPair as _};

use crate::{signatures::Signature, Algorithm, Error};

/// A cryptographic key pair for digitally signing data.
pub trait KeyPair: Sized {
    /// Signs a JSON object.
    ///
    /// # Parameters
    ///
    /// * message: An arbitrary series of bytes to sign.
    fn sign(&self, message: &[u8]) -> Signature;
}

/// An Ed25519 key pair.
pub struct Ed25519KeyPair {
    /// Ring's Keypair type
    keypair: RingEd25519KeyPair,

    /// The version of the key pair.
    version: String,
}

impl Ed25519KeyPair {
    /// Initializes a new key pair.
    ///
    /// # Parameters
    ///
    /// * document: PKCS8-formatted bytes containing the private & public keys.
    /// * version: The "version" of the key used for this signature. Versions are used as an
    ///   identifier to distinguish signatures generated from different keys but using the same
    ///   algorithm on the same homeserver.
    ///
    /// # Errors
    ///
    /// Returns an error if the public and private keys provided are invalid for the implementing
    /// algorithm.
    pub fn new(document: &[u8], version: String) -> Result<Self, Error> {
        let keypair = RingEd25519KeyPair::from_pkcs8(document)
            .map_err(|error| Error::new(error.to_string()))?;

        Ok(Self { keypair, version })
    }

    /// Generates a new key pair.
    ///
    /// # Returns
    ///
    /// Returns a Vec<u8> representing a pkcs8-encoded private/public keypair
    ///
    /// # Errors
    ///
    /// Returns an error if the generation failed.
    pub fn generate() -> Result<Vec<u8>, Error> {
        let document = RingEd25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new())
            .map_err(|e| Error::new(e.to_string()))?;

        Ok(document.as_ref().to_vec())
    }

    /// Returns the version string for this keypair.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns the public key.
    pub fn public_key(&self) -> &[u8] {
        self.keypair.public_key().as_ref()
    }
}

impl KeyPair for Ed25519KeyPair {
    fn sign(&self, message: &[u8]) -> Signature {
        Signature {
            algorithm: Algorithm::Ed25519,
            signature: self.keypair.sign(message).as_ref().to_vec(),
            version: self.version.clone(),
        }
    }
}

impl Debug for Ed25519KeyPair {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter
            .debug_struct("Ed25519KeyPair")
            .field("public_key", &self.keypair.public_key())
            .field("version", &self.version)
            .finish()
    }
}

/// A map from entity names to sets of public keys for that entity.
///
/// "Entity" is generally a homeserver, e.g. "example.com".
pub type PublicKeyMap = BTreeMap<String, PublicKeySet>;

/// A set of public keys for a single homeserver.
///
/// This is represented as a map from key ID to Base64-encoded signature.
pub type PublicKeySet = BTreeMap<String, String>;

#[cfg(test)]
mod tests {
    use super::Ed25519KeyPair;

    #[test]
    fn generate_key() {
        Ed25519KeyPair::generate().unwrap();
    }
}
