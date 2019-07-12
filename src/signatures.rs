//! Digital signatures and collections of signatures.

use std::collections::HashMap;

use base64::{encode_config, STANDARD_NO_PAD};

use crate::{Algorithm, Error};

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
    /// Returns an error if the key identifier is invalid.
    pub fn new(id: &str, bytes: &[u8]) -> Result<Self, Error> {
        let (algorithm, version) = split_id(id).map_err(|split_error| match split_error {
            SplitError::InvalidLength(length) => Error::new(format!("malformed signature ID: expected exactly 2 segment separated by a colon, found {}", length)),
            SplitError::InvalidVersion(version) => Error::new(format!("malformed signature ID: expected version to contain only characters in the character set `[a-zA-Z0-9_]`, found `{}`", version)),
            SplitError::UnknownAlgorithm(algorithm) => {
                Error::new(format!("unknown algorithm: {}", algorithm))
            }
        })?;

        Ok(Self {
            algorithm,
            signature: bytes.to_vec(),
            version,
        })
    }

    /// The algorithm used to generate the signature.
    pub fn algorithm(&self) -> Algorithm {
        self.algorithm
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

/// A map from entity names to sets of digital signatures created by that entity.
///
/// "Entity" is generally a homeserver, e.g. "example.com".
pub type SignatureMap = HashMap<String, SignatureSet>;

/// A set of digital signatures created by a single homeserver.
///
/// This is represented as a map from signing key ID to Base64-encoded signature.
pub type SignatureSet = HashMap<String, String>;

/// An error when trying to extract the algorithm and version from a key identifier.
#[derive(Clone, Debug, PartialEq)]
enum SplitError<'a> {
    /// The signature's ID does not have exactly two components separated by a colon.
    InvalidLength(usize),
    /// The signature's ID contains invalid characters in its version.
    InvalidVersion(&'a str),
    /// The signature uses an unknown algorithm.
    UnknownAlgorithm(&'a str),
}

/// Extract the algorithm and version from a key identifier.
fn split_id(id: &str) -> Result<(Algorithm, String), SplitError<'_>> {
    /// The length of a valid signature ID.
    const SIGNATURE_ID_LENGTH: usize = 2;

    let signature_id: Vec<&str> = id.split(':').collect();

    let signature_id_length = signature_id.len();

    if signature_id_length != SIGNATURE_ID_LENGTH {
        return Err(SplitError::InvalidLength(signature_id_length));
    }

    let version = signature_id[1];

    let invalid_character_index = version.find(|ch| {
        !((ch >= 'a' && ch <= 'z')
            || (ch >= 'A' && ch <= 'Z')
            || (ch >= '0' && ch <= '9')
            || ch == '_')
    });

    if invalid_character_index.is_some() {
        return Err(SplitError::InvalidVersion(version));
    }

    let algorithm_input = signature_id[0];

    let algorithm = match algorithm_input {
        "ed25519" => Algorithm::Ed25519,
        algorithm => return Err(SplitError::UnknownAlgorithm(algorithm)),
    };

    Ok((algorithm, signature_id[1].to_string()))
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
