//! Types for the `ed25519` signing algorithm.

use std::fmt;

use ed25519_dalek::{
    PUBLIC_KEY_LENGTH, SecretKey, Signer, SigningKey, Verifier as _,
    VerifyingKey as Ed25519VerifyingKey, ed25519::Signature as Ed25519Signature,
    pkcs8::ALGORITHM_OID,
};
use pkcs8::{
    DecodePrivateKey, EncodePrivateKey, ObjectIdentifier, PrivateKeyInfo, der::zeroize::Zeroizing,
};
use ruma_common::{SigningKeyAlgorithm, SigningKeyId};
use thiserror::Error;

use crate::{KeyPair, Signature, verify::Verifier};

#[cfg(feature = "ring-compat")]
mod compat;

/// An Ed25519 key pair.
pub struct Ed25519KeyPair {
    signing_key: SigningKey,
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
    ) -> Result<Self, Ed25519KeyPairParseError> {
        if oid != ALGORITHM_OID {
            return Err(Ed25519KeyPairParseError::InvalidOid {
                expected: ALGORITHM_OID,
                found: oid,
            });
        }

        let secret_key = Self::correct_privkey_from_octolet(privkey)?;
        let signing_key = SigningKey::from_bytes(secret_key);

        if let Some(oak_key) = pubkey {
            // If the document had a public key, we're verifying it.
            let verifying_key = signing_key.verifying_key();

            if oak_key != verifying_key.as_bytes() {
                return Err(Ed25519KeyPairParseError::PublicKeyMismatch {
                    derived: verifying_key.as_bytes().to_vec(),
                    parsed: oak_key.to_owned(),
                });
            }
        }

        Ok(Self { signing_key, version })
    }

    /// Initializes a new key pair.
    ///
    /// # Parameters
    ///
    /// * `document`: PKCS#8 v1/v2 DER-formatted document containing the private (and optionally
    ///   public) key.
    /// * `version`: The "version" of the key used for this signature. Versions are used as an
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
    pub fn from_der(document: &[u8], version: String) -> Result<Self, Ed25519KeyPairParseError> {
        #[cfg(feature = "ring-compat")]
        use self::compat::CompatibleDocument;

        let signing_key;

        #[cfg(feature = "ring-compat")]
        {
            signing_key = match CompatibleDocument::from_bytes(document) {
                CompatibleDocument::WellFormed(bytes) => SigningKey::from_pkcs8_der(bytes)?,
                CompatibleDocument::CleanedFromRing(vec) => SigningKey::from_pkcs8_der(&vec)?,
            }
        }
        #[cfg(not(feature = "ring-compat"))]
        {
            signing_key = SigningKey::from_pkcs8_der(document)?;
        }

        Ok(Self { signing_key, version })
    }

    /// Constructs a key pair from [`pkcs8::PrivateKeyInfo`].
    pub fn from_pkcs8_oak(
        oak: PrivateKeyInfo<'_>,
        version: String,
    ) -> Result<Self, Ed25519KeyPairParseError> {
        Self::new(oak.algorithm.oid, oak.private_key, oak.public_key, version)
    }

    /// Constructs a key pair from [`pkcs8::PrivateKeyInfo`].
    pub fn from_pkcs8_pki(
        oak: PrivateKeyInfo<'_>,
        version: String,
    ) -> Result<Self, Ed25519KeyPairParseError> {
        Self::new(oak.algorithm.oid, oak.private_key, None, version)
    }

    /// PKCS#8's "private key" is not yet actually the entire key,
    /// so convert it if it is wrongly formatted.
    ///
    /// See [RFC 8310 10.3](https://datatracker.ietf.org/doc/html/rfc8410#section-10.3) for more details
    fn correct_privkey_from_octolet(key: &[u8]) -> Result<&SecretKey, Ed25519KeyPairParseError> {
        if key.len() == 34 && key[..2] == [0x04, 0x20] {
            Ok(key[2..].try_into().unwrap())
        } else {
            key.try_into().map_err(|_| Ed25519KeyPairParseError::InvalidSecretKeyLength {
                expected: ed25519_dalek::SECRET_KEY_LENGTH,
                found: key.len(),
            })
        }
    }

    /// Generates a new key pair.
    ///
    /// # Returns
    ///
    /// Returns a `Vec<u8>` representing a DER-encoded PKCS#8 v2 document (with public key).
    ///
    /// # Errors
    ///
    /// Returns an error if the generation failed.
    pub fn generate() -> Result<Zeroizing<Vec<u8>>, Ed25519KeyPairParseError> {
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        Ok(signing_key.to_pkcs8_der()?.to_bytes())
    }

    /// Returns the version string for this keypair.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns the public key.
    pub fn public_key(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        self.signing_key.verifying_key().to_bytes()
    }
}

impl KeyPair for Ed25519KeyPair {
    fn sign(&self, message: &[u8]) -> Signature {
        Signature {
            key_id: SigningKeyId::from_parts(
                SigningKeyAlgorithm::Ed25519,
                self.version.as_str().into(),
            ),
            signature: self.signing_key.sign(message).to_bytes().to_vec(),
        }
    }
}

impl fmt::Debug for Ed25519KeyPair {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Ed25519KeyPair")
            .field("verifying_key", &self.signing_key.verifying_key().as_bytes())
            .field("version", &self.version)
            .finish()
    }
}

/// An error encountered when constructing an [`Ed25519KeyPair`] from its constituent parts.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Ed25519KeyPairParseError {
    /// The ASN.1 Object Identifier on a PKCS#8 document doesn't match the expected one.
    ///
    /// This can happen when the document describes a RSA key, while an ed25519 key was expected.
    #[error("algorithm OID does not match ed25519 algorithm: expected {expected}, found {found}")]
    InvalidOid {
        /// The expected OID.
        expected: ObjectIdentifier,

        /// The OID that was found instead.
        found: ObjectIdentifier,
    },

    /// The length of the ed25519 secret key is invalid.
    #[error("invalid ed25519 secret key length: expected {expected}, found {found}")]
    InvalidSecretKeyLength {
        /// The expected length of the secret key.
        expected: usize,

        /// The actual size of the secret key.
        found: usize,
    },

    /// The public key found in a PKCS#8 v2 document doesn't match the public key derived from its
    /// private key.
    #[error("PKCS#8 Document public key does not match public key derived from private key: derived {0:X?} (len {}), parsed {1:X?} (len {})", .derived.len(), .parsed.len())]
    PublicKeyMismatch {
        /// The key derived from the private key.
        derived: Vec<u8>,

        /// The key found in the document.
        parsed: Vec<u8>,
    },

    /// An error occurred when parsing a PKCS#8 document.
    #[error("invalid PKCS#8 document: {0}")]
    Pkcs8(#[from] pkcs8::Error),
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

#[cfg(test)]
mod tests {
    use super::Ed25519KeyPair;

    const WELL_FORMED_DOC: &[u8] = &[
        0x30, 0x72, 0x02, 0x01, 0x01, 0x30, 0x05, 0x06, 0x03, 0x2B, 0x65, 0x70, 0x04, 0x22, 0x04,
        0x20, 0xD4, 0xEE, 0x72, 0xDB, 0xF9, 0x13, 0x58, 0x4A, 0xD5, 0xB6, 0xD8, 0xF1, 0xF7, 0x69,
        0xF8, 0xAD, 0x3A, 0xFE, 0x7C, 0x28, 0xCB, 0xF1, 0xD4, 0xFB, 0xE0, 0x97, 0xA8, 0x8F, 0x44,
        0x75, 0x58, 0x42, 0xA0, 0x1F, 0x30, 0x1D, 0x06, 0x0A, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D,
        0x01, 0x09, 0x09, 0x14, 0x31, 0x0F, 0x0C, 0x0D, 0x43, 0x75, 0x72, 0x64, 0x6C, 0x65, 0x20,
        0x43, 0x68, 0x61, 0x69, 0x72, 0x73, 0x81, 0x21, 0x00, 0x19, 0xBF, 0x44, 0x09, 0x69, 0x84,
        0xCD, 0xFE, 0x85, 0x41, 0xBA, 0xC1, 0x67, 0xDC, 0x3B, 0x96, 0xC8, 0x50, 0x86, 0xAA, 0x30,
        0xB6, 0xB6, 0xCB, 0x0C, 0x5C, 0x38, 0xAD, 0x70, 0x31, 0x66, 0xE1,
    ];

    const WELL_FORMED_PUBKEY: &[u8] = &[
        0x19, 0xBF, 0x44, 0x09, 0x69, 0x84, 0xCD, 0xFE, 0x85, 0x41, 0xBA, 0xC1, 0x67, 0xDC, 0x3B,
        0x96, 0xC8, 0x50, 0x86, 0xAA, 0x30, 0xB6, 0xB6, 0xCB, 0x0C, 0x5C, 0x38, 0xAD, 0x70, 0x31,
        0x66, 0xE1,
    ];

    #[test]
    fn generate_key() {
        Ed25519KeyPair::generate().unwrap();
    }

    #[test]
    fn well_formed_key() {
        let keypair = Ed25519KeyPair::from_der(WELL_FORMED_DOC, "".to_owned()).unwrap();

        assert_eq!(keypair.public_key(), WELL_FORMED_PUBKEY);
    }

    #[cfg(feature = "ring-compat")]
    mod ring_compat {
        use super::Ed25519KeyPair;

        const RING_DOC: &[u8] = &[
            0x30, 0x53, 0x02, 0x01, 0x01, 0x30, 0x05, 0x06, 0x03, 0x2B, 0x65, 0x70, 0x04, 0x22,
            0x04, 0x20, 0x61, 0x9E, 0xD8, 0x25, 0xA6, 0x1D, 0x32, 0x29, 0xD7, 0xD8, 0x22, 0x03,
            0xC6, 0x0E, 0x37, 0x48, 0xE9, 0xC9, 0x11, 0x96, 0x3B, 0x03, 0x15, 0x94, 0x19, 0x3A,
            0x86, 0xEC, 0xE6, 0x2D, 0x73, 0xC0, 0xA1, 0x23, 0x03, 0x21, 0x00, 0x3D, 0xA6, 0xC8,
            0xD1, 0x76, 0x2F, 0xD6, 0x49, 0xB8, 0x4F, 0xF6, 0xC6, 0x1D, 0x04, 0xEA, 0x4A, 0x70,
            0xA8, 0xC9, 0xF0, 0x8F, 0x96, 0x7F, 0x6B, 0xD7, 0xDA, 0xE5, 0x2E, 0x88, 0x8D, 0xBA,
            0x3E,
        ];

        const RING_PUBKEY: &[u8] = &[
            0x3D, 0xA6, 0xC8, 0xD1, 0x76, 0x2F, 0xD6, 0x49, 0xB8, 0x4F, 0xF6, 0xC6, 0x1D, 0x04,
            0xEA, 0x4A, 0x70, 0xA8, 0xC9, 0xF0, 0x8F, 0x96, 0x7F, 0x6B, 0xD7, 0xDA, 0xE5, 0x2E,
            0x88, 0x8D, 0xBA, 0x3E,
        ];

        #[test]
        fn ring_key() {
            let keypair = Ed25519KeyPair::from_der(RING_DOC, "".to_owned()).unwrap();

            assert_eq!(keypair.public_key(), RING_PUBKEY);
        }
    }
}
