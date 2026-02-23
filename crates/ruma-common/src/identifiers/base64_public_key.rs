use ruma_macros::ruma_id;

use super::{IdParseError, KeyName};
use crate::serde::{Base64, Base64DecodeError, base64::Standard};

/// A public key encoded using unpadded base64, used as an identifier for [cross-signing] keys.
///
/// This string is validated using the set `[a-zA-Z0-9+/=]`, but it is not validated to be decodable
/// as base64. This type is provided simply for its semantic value.
///
/// [cross-signing]: https://spec.matrix.org/latest/client-server-api/#cross-signing
#[ruma_id(validate = ruma_identifiers_validation::base64_public_key::validate)]
pub struct Base64PublicKey;

impl Base64PublicKey {
    /// Construct a new `Base64PublicKey` by encoding the given bytes using unpadded base64.
    pub fn with_bytes<B: AsRef<[u8]>>(bytes: B) -> Self {
        Base64::<Standard, B>::new(bytes).into()
    }
}

impl KeyName for Base64PublicKey {
    fn validate(s: &str) -> Result<(), IdParseError> {
        ruma_identifiers_validation::base64_public_key::validate(s)
    }
}

impl<B: AsRef<[u8]>> From<Base64<Standard, B>> for Base64PublicKey {
    fn from(value: Base64<Standard, B>) -> Self {
        value.to_string().try_into().unwrap_or_else(|_| unreachable!())
    }
}

impl TryFrom<&Base64PublicKey> for Base64<Standard, Vec<u8>> {
    type Error = Base64DecodeError;

    fn try_from(value: &Base64PublicKey) -> Result<Self, Self::Error> {
        Base64::parse(value)
    }
}

impl TryFrom<Base64PublicKey> for Base64<Standard, Vec<u8>> {
    type Error = Base64DecodeError;

    fn try_from(value: Base64PublicKey) -> Result<Self, Self::Error> {
        Base64::parse(value)
    }
}

#[cfg(test)]
mod tests {
    use super::Base64PublicKey;

    #[test]
    fn valid_string() {
        Base64PublicKey::try_from("base64+master+public+key").unwrap();
    }

    #[test]
    fn invalid_string() {
        Base64PublicKey::try_from("not@base@64").unwrap_err();
    }

    #[test]
    fn constructor() {
        _ = Base64PublicKey::with_bytes(b"self-signing master public key");
    }
}
