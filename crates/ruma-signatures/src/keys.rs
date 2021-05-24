//! Public and private key pairs.

use std::{
    collections::BTreeMap,
    fmt::{Debug, Formatter, Result as FmtResult},
};

use ed25519_dalek::{ExpandedSecretKey, PublicKey, SecretKey};

use pkcs8::{
    der::{Decodable, Encodable},
    AlgorithmIdentifier, ObjectIdentifier, OneAsymmetricKey, PrivateKeyInfo,
};

use crate::{signatures::Signature, Algorithm, Error, ParseError};

/// A cryptographic key pair for digitally signing data.
pub trait KeyPair: Sized {
    /// Signs a JSON object.
    ///
    /// # Parameters
    ///
    /// * message: An arbitrary series of bytes to sign.
    fn sign(&self, message: &[u8]) -> Signature;
}

pub const ED25519_OID: ObjectIdentifier = ObjectIdentifier::new("1.3.101.112");

/// An Ed25519 key pair.
pub struct Ed25519KeyPair {
    extended_privkey: ExpandedSecretKey,

    pubkey: PublicKey,

    /// The specific name of the key pair.
    version: String,
}

impl Ed25519KeyPair {
    /// Create a key pair from its constituent parts.
    pub fn new(
        oid: ObjectIdentifier,
        privkey: &[u8],
        pubkey: Option<&[u8]>,
        version: String,
    ) -> Result<Self, Error> {
        if oid != ED25519_OID {
            return Err(ParseError::Oid { expected: ED25519_OID, found: oid }.into());
        }

        let secret_key = SecretKey::from_bytes(Self::correct_privkey_from_octolet(privkey))
            .map_err(ParseError::SecretKey)?;

        let derived_pubkey = PublicKey::from(&secret_key);

        if let Some(oak_key) = pubkey {
            // If the document had a public key, we're verifying it.

            if oak_key != derived_pubkey.as_bytes() {
                return Err(ParseError::derived_vs_parsed_mismatch(
                    oak_key,
                    derived_pubkey.as_bytes().to_vec(),
                ));
            }
        }

        Ok(Self {
            extended_privkey: ExpandedSecretKey::from(&secret_key),
            pubkey: derived_pubkey,
            version,
        })
    }

    /// Initializes a new key pair.
    ///
    /// # Parameters
    ///
    /// * document: PKCS#8 v1/v2 DER-formatted document containing the private (and optionally
    ///   public) key.
    /// * version: The "version" of the key used for this signature. Versions are used as an
    ///   identifier to distinguish signatures generated from different keys but using the same
    ///   algorithm on the same homeserver.
    ///
    /// # Errors
    ///
    /// Returns an error if the public and private keys provided are invalid for the implementing
    /// algorithm.
    ///
    /// Returns an error when the PKCS#8 document had a public key, but it doesn't match the one
    /// generated from the private key. This is a fallback and extra validation against
    /// corruption or
    pub fn from_der(document: &[u8], version: String) -> Result<Self, Error> {
        let oak = OneAsymmetricKey::from_der(document).map_err(Error::DerParse)?;

        Self::from_pkcs8_oak(oak, version)
    }

    /// Constructs a key pair from [`pkcs8::OneAsymmetricKey`].
    pub fn from_pkcs8_oak(oak: OneAsymmetricKey<'_>, version: String) -> Result<Self, Error> {
        Self::new(oak.algorithm.oid, oak.private_key, oak.public_key, version)
    }

    /// Constructs a key pair from [`pkcs8::PrivateKeyInfo`].
    pub fn from_pkcs8_pki(oak: PrivateKeyInfo<'_>, version: String) -> Result<Self, Error> {
        Self::new(oak.algorithm.oid, oak.private_key, None, version)
    }

    /// PKCS#8's "private key" is not yet actually the entire key,
    /// so convert it if it is wrongly formatted.
    ///
    /// See [RFC 8310 10.3](https://datatracker.ietf.org/doc/html/rfc8410#section-10.3) for more details
    fn correct_privkey_from_octolet(key: &[u8]) -> &[u8] {
        if key.len() == 34 && key[..2] == [0x04, 0x20] {
            &key[2..]
        } else {
            key
        }
    }

    /// Generates a new key pair.
    ///
    /// # Returns
    ///
    /// Returns a Vec<u8> representing a DER-encoded PKCS#8 v2 document (with public key)
    ///
    /// # Errors
    ///
    /// Returns an error if the generation failed.
    pub fn generate() -> Result<Vec<u8>, Error> {
        let secret = SecretKey::generate(&mut rand::rngs::OsRng);

        let public = PublicKey::from(&secret);

        // Convert into nested OCTAL STRING
        // Per: https://datatracker.ietf.org/doc/html/rfc8410#section-10.3
        let mut private: Vec<u8> = vec![0x04, 0x20];
        private.extend_from_slice(secret.as_bytes());

        let oak = OneAsymmetricKey {
            algorithm: AlgorithmIdentifier { oid: ED25519_OID, parameters: None },
            private_key: private.as_ref(),
            public_key: Some(public.as_bytes()),
        };

        oak.to_vec().map_err(Error::DerParse)
    }

    /// Returns the version string for this keypair.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns the public key.
    pub fn public_key(&self) -> &[u8] {
        self.pubkey.as_ref()
    }
}

impl KeyPair for Ed25519KeyPair {
    fn sign(&self, message: &[u8]) -> Signature {
        Signature {
            algorithm: Algorithm::Ed25519,
            signature: self.extended_privkey.sign(message, &self.pubkey).as_ref().to_vec(),
            version: self.version.clone(),
        }
    }
}

impl Debug for Ed25519KeyPair {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter
            .debug_struct("Ed25519KeyPair")
            .field("public_key", &self.pubkey.as_bytes())
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

    const RING_DOC: &[u8] = &[
        0x30, 0x53, 0x02, 0x01, 0x01, 0x30, 0x05, 0x06, 0x03, 0x2B, 0x65, 0x70, 0x04, 0x22, 0x04,
        0x20, 0x61, 0x9E, 0xD8, 0x25, 0xA6, 0x1D, 0x32, 0x29, 0xD7, 0xD8, 0x22, 0x03, 0xC6, 0x0E,
        0x37, 0x48, 0xE9, 0xC9, 0x11, 0x96, 0x3B, 0x03, 0x15, 0x94, 0x19, 0x3A, 0x86, 0xEC, 0xE6,
        0x2D, 0x73, 0xC0, 0xA1, 0x23, 0x03, 0x21, 0x00, 0x3D, 0xA6, 0xC8, 0xD1, 0x76, 0x2F, 0xD6,
        0x49, 0xB8, 0x4F, 0xF6, 0xC6, 0x1D, 0x04, 0xEA, 0x4A, 0x70, 0xA8, 0xC9, 0xF0, 0x8F, 0x96,
        0x7F, 0x6B, 0xD7, 0xDA, 0xE5, 0x2E, 0x88, 0x8D, 0xBA, 0x3E,
    ];

    const RING_PUBKEY: &[u8] = &[
        0x3D, 0xA6, 0xC8, 0xD1, 0x76, 0x2F, 0xD6, 0x49, 0xB8, 0x4F, 0xF6, 0xC6, 0x1D, 0x04, 0xEA,
        0x4A, 0x70, 0xA8, 0xC9, 0xF0, 0x8F, 0x96, 0x7F, 0x6B, 0xD7, 0xDA, 0xE5, 0x2E, 0x88, 0x8D,
        0xBA, 0x3E,
    ];

    #[test]
    fn generate_key() {
        Ed25519KeyPair::generate().unwrap();
    }

    #[test]
    fn ring_key() {
        let keypair = Ed25519KeyPair::from_der(RING_DOC, "".to_string()).unwrap();

        assert_eq!(keypair.pubkey.as_bytes(), RING_PUBKEY);
    }
}
