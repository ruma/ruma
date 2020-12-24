//! Digital signatures and collections of signatures.

use base64::{encode_config, STANDARD_NO_PAD};

use crate::{split_id, Algorithm, Error, SplitError};

/// A digital signature.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Signature {
    /// The cryptographic algorithm that generated this signature.
    pub algorithm: Algorithm,

    /// The signature data.
    pub signature: Vec<u8>,

    /// The "version" of the key identifier for the public key used to generate this signature.
    pub version: String,
}

impl Signature {
    /// Creates a signature from raw bytes.
    ///
    /// While a signature can be created directly using struct literal syntax, this constructor can
    /// be used to automatically determine the algorithm and version from a key identifier in the
    /// form *algorithm:version*, e.g. "ed25519:1".
    ///
    /// This constructor will ensure that the version does not contain characters that violate the
    /// guidelines in the specification. Because it may be necessary to represent signatures with
    /// versions that don't adhere to these guidelines, it's possible to simply use the struct
    /// literal syntax to construct a `Signature` with an arbitrary key.
    ///
    /// # Parameters
    ///
    /// * id: A key identifier, e.g. "ed25519:1".
    /// * bytes: The digital signature, as a series of bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// * The key ID specifies an unknown algorithm.
    /// * The key ID is malformed.
    /// * The key ID contains a version with invalid characters.
    pub fn new(id: &str, bytes: &[u8]) -> Result<Self, Error> {
        let (algorithm, version) = split_id(id).map_err(|split_error| match split_error {
            SplitError::InvalidLength(length) => Error::new(format!(
                "malformed signature ID: expected exactly \
                 2 segment separated by a colon, found {}",
                length
            )),
            SplitError::InvalidVersion(version) => Error::new(format!(
                "malformed signature ID: expected version to contain only \
                 characters in the character set `[a-zA-Z0-9_]`, found `{}`",
                version
            )),
            SplitError::UnknownAlgorithm(algorithm) => {
                Error::new(format!("unknown algorithm: {}", algorithm))
            }
        })?;

        Ok(Self { algorithm, signature: bytes.to_vec(), version })
    }

    /// The algorithm used to generate the signature.
    pub fn algorithm(&self) -> &Algorithm {
        &self.algorithm
    }

    /// The raw bytes of the signature.
    pub fn as_bytes(&self) -> &[u8] {
        self.signature.as_slice()
    }

    /// A Base64 encoding of the signature.
    ///
    /// Uses the standard character set with no padding.
    pub fn base64(&self) -> String {
        encode_config(self.signature.as_slice(), STANDARD_NO_PAD)
    }

    /// The key identifier, a string containing the signature algorithm and the key "version"
    /// separated by a colon, e.g. "ed25519:1".
    pub fn id(&self) -> String {
        format!("{}:{}", self.algorithm, self.version)
    }

    /// The "version" of the key used for this signature.
    ///
    /// Versions are used as an identifier to distinguish signatures generated from different keys
    /// but using the same algorithm on the same homeserver.
    pub fn version(&self) -> &str {
        &self.version
    }
}

#[cfg(test)]
mod tests {
    use super::Signature;

    #[test]
    fn valid_key_id() {
        assert!(Signature::new("ed25519:abcdef", &[]).is_ok());
    }

    #[test]
    fn invalid_valid_key_id_length() {
        assert!(Signature::new("ed25519:abcdef:123456", &[]).is_err());
    }

    #[test]
    fn invalid_key_id_version() {
        assert!(Signature::new("ed25519:abc!def", &[]).is_err());
    }

    #[test]
    fn invalid_key_id_algorithm() {
        assert!(Signature::new("foobar:abcdef", &[]).is_err());
    }
}
