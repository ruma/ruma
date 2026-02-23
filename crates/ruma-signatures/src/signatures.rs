//! Digital signatures and collections of signatures.

use ruma_common::{
    AnyKeyName, IdParseError, SigningKeyAlgorithm, SigningKeyId,
    serde::{Base64, base64::Standard},
};

/// A digital signature.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Signature {
    /// The ID of the key used to generate this signature.
    pub(crate) key_id: SigningKeyId<AnyKeyName>,

    /// The signature data.
    pub(crate) signature: Vec<u8>,
}

impl Signature {
    /// Creates a signature from raw bytes.
    ///
    /// This constructor will ensure that the key ID has the correct `algorithm:key_name` format.
    ///
    /// # Parameters
    ///
    /// * `id`: A key identifier, e.g. `ed25519:1`.
    /// * `bytes`: The digital signature, as a series of bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// * The key ID is malformed.
    pub fn new(id: &str, bytes: &[u8]) -> Result<Self, IdParseError> {
        let key_id = SigningKeyId::<AnyKeyName>::parse(id)?;

        Ok(Self { key_id, signature: bytes.to_vec() })
    }

    /// The algorithm used to generate the signature.
    pub fn algorithm(&self) -> SigningKeyAlgorithm {
        self.key_id.algorithm()
    }

    /// The raw bytes of the signature.
    pub fn as_bytes(&self) -> &[u8] {
        self.signature.as_slice()
    }

    /// A base64 encoding of the signature.
    ///
    /// Uses the standard character set with no padding.
    pub fn base64(&self) -> String {
        Base64::<Standard, _>::new(self.signature.as_slice()).encode()
    }

    /// The key identifier, a string containing the signature algorithm and the key "version"
    /// separated by a colon, e.g. `ed25519:1`.
    pub fn id(&self) -> String {
        self.key_id.to_string()
    }

    /// The "version" of the key used for this signature.
    ///
    /// Versions are used as an identifier to distinguish signatures generated from different keys
    /// but using the same algorithm on the same homeserver.
    pub fn version(&self) -> &str {
        self.key_id.key_name_str()
    }

    /// Split this `Signature` into its key identifier and bytes.
    pub fn into_parts(self) -> (SigningKeyId<AnyKeyName>, Vec<u8>) {
        (self.key_id, self.signature)
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::SigningKeyAlgorithm;

    use super::Signature;

    #[test]
    fn valid_key_id() {
        let signature = Signature::new("ed25519:abcdef", &[]).unwrap();
        assert_eq!(signature.algorithm(), SigningKeyAlgorithm::Ed25519);
        assert_eq!(signature.version(), "abcdef");
    }

    #[test]
    fn unknown_key_id_algorithm() {
        let signature = Signature::new("foobar:abcdef", &[]).unwrap();
        assert_eq!(signature.algorithm().as_str(), "foobar");
        assert_eq!(signature.version(), "abcdef");
    }

    #[test]
    fn invalid_key_id_format() {
        Signature::new("ed25519", &[]).unwrap_err();
    }
}
