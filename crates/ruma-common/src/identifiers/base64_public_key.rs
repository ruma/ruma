use ruma_macros::IdZst;

use super::{IdParseError, KeyName};
use crate::serde::{base64::Standard, Base64, Base64DecodeError};

/// A public key encoded using unpadded base64, used as an identifier for [cross-signing] keys.
///
/// This string is validated using the set `[a-zA-Z0-9+/=]`, but it is not validated to be decodable
/// as base64. This type is provided simply for its semantic value.
///
/// [cross-signing]: https://spec.matrix.org/latest/client-server-api/#cross-signing
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdZst)]
#[ruma_id(validate = ruma_identifiers_validation::base64_public_key::validate)]
pub struct Base64PublicKey(str);

impl OwnedBase64PublicKey {
    /// Construct a new `OwnedBase64PublicKey` by encoding the given bytes using unpadded base64.
    pub fn with_bytes<B: AsRef<[u8]>>(bytes: B) -> OwnedBase64PublicKey {
        Base64::<Standard, B>::new(bytes).into()
    }
}

impl KeyName for Base64PublicKey {
    fn validate(s: &str) -> Result<(), IdParseError> {
        ruma_identifiers_validation::base64_public_key::validate(s)
    }
}

impl KeyName for OwnedBase64PublicKey {
    fn validate(s: &str) -> Result<(), IdParseError> {
        ruma_identifiers_validation::base64_public_key::validate(s)
    }
}

impl<B: AsRef<[u8]>> From<Base64<Standard, B>> for OwnedBase64PublicKey {
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

impl TryFrom<&OwnedBase64PublicKey> for Base64<Standard, Vec<u8>> {
    type Error = Base64DecodeError;

    fn try_from(value: &OwnedBase64PublicKey) -> Result<Self, Self::Error> {
        Base64::parse(value)
    }
}

impl TryFrom<OwnedBase64PublicKey> for Base64<Standard, Vec<u8>> {
    type Error = Base64DecodeError;

    fn try_from(value: OwnedBase64PublicKey) -> Result<Self, Self::Error> {
        Base64::parse(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{Base64PublicKey, OwnedBase64PublicKey};

    #[test]
    fn valid_string() {
        <&Base64PublicKey>::try_from("base64+master+public+key").unwrap();
    }

    #[test]
    fn invalid_string() {
        <&Base64PublicKey>::try_from("not@base@64").unwrap_err();
    }

    #[test]
    fn constructor() {
        _ = OwnedBase64PublicKey::with_bytes(b"self-signing master public key");
    }
}
