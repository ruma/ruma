//! Common types for [third-party invites].
//!
//! [third-party invites]: https://spec.matrix.org/latest/client-server-api/#third-party-invites

use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::serde::{
    base64::{Standard, UrlSafe},
    Base64, Base64DecodeError,
};

/// A base64-encoded public key from an [identity server].
///
/// This type supports both standard and URL-safe base64, for [compatibility with Sydent].
///
/// No validation is done on the inner string during deserialization, this type is used for its
/// semantic value and for providing a helper to decode it.
///
/// The key can be decoded by calling [`IdentityServerBase64PublicKey::decode()`].
///
/// [identity server]: https://spec.matrix.org/latest/identity-service-api/
/// [compatibility with Sydent]: https://github.com/matrix-org/sydent/issues/593
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[allow(clippy::exhaustive_structs)]
pub struct IdentityServerBase64PublicKey(pub String);

impl IdentityServerBase64PublicKey {
    /// Construct a new `IdentityServerBase64PublicKey` by encoding the given key using unpadded
    /// standard base64.
    pub fn new(bytes: &[u8]) -> Self {
        Self(Base64::<Standard, &[u8]>::new(bytes).encode())
    }

    /// Try to decode this base64-encoded string.
    ///
    /// This will try to detect the proper alphabet to use for decoding, between
    /// the standard and the URL-safe alphabet.
    pub fn decode(&self) -> Result<Vec<u8>, Base64DecodeError> {
        let is_url_safe_alphabet = self.0.contains(['-', '_']);

        if is_url_safe_alphabet {
            Ok(Base64::<UrlSafe>::parse(&self.0)?.into_inner())
        } else {
            Ok(Base64::<Standard>::parse(&self.0)?.into_inner())
        }
    }
}

impl From<String> for IdentityServerBase64PublicKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for IdentityServerBase64PublicKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for IdentityServerBase64PublicKey {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl PartialEq<String> for IdentityServerBase64PublicKey {
    fn eq(&self, other: &String) -> bool {
        self.0.eq(other)
    }
}

impl<'a> PartialEq<&'a str> for IdentityServerBase64PublicKey {
    fn eq(&self, other: &&'a str) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<str> for IdentityServerBase64PublicKey {
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

#[cfg(test)]
mod tests {
    use super::IdentityServerBase64PublicKey;

    #[test]
    fn identity_server_base64_public_key_encode_then_decode() {
        let original = b"foobar";
        let encoded = IdentityServerBase64PublicKey::new(original);
        assert_eq!(encoded, "Zm9vYmFy");
        assert_eq!(encoded.decode().unwrap(), original);
    }

    #[test]
    fn identity_server_base64_public_key_decode_standard_and_url_safe() {
        let original = &[60, 98, 62, 77, 68, 78, 60, 47, 98, 62];
        let standard_base64 = IdentityServerBase64PublicKey("PGI+TUROPC9iPg".to_owned());
        assert_eq!(standard_base64.decode().unwrap(), original);
        let urlsafe_base64 = IdentityServerBase64PublicKey("PGI-TUROPC9iPg".to_owned());
        assert_eq!(urlsafe_base64.decode().unwrap(), original);
    }
}
