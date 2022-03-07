//! Client secret identifier.

/// A client secret.
///
/// Client secrets in Matrix are opaque character sequences of `[0-9a-zA-Z.=_-]`. Their length must
/// must not exceed 255 characters.
///
/// You can create one from a string (using `ClientSecret::parse()`) but the recommended way is to
/// use `ClientSecret::new()` to generate a random one. If that function is not available for you,
/// you need to activate this crate's `rand` Cargo feature.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClientSecret(str);

impl ClientSecret {
    /// Creates a random client secret.
    ///
    /// This will currently be a UUID without hyphens, but no guarantees are made about the
    /// structure of client secrets generated from this function.
    #[cfg(feature = "rand")]
    pub fn new() -> Box<Self> {
        let id = uuid::Uuid::new_v4();
        Self::from_owned(id.to_simple().to_string().into_boxed_str())
    }
}

opaque_identifier_validated!(ClientSecret, ruma_identifiers_validation::client_secret::validate);

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::ClientSecret;

    #[test]
    fn valid_secret() {
        assert!(<&ClientSecret>::try_from("this_=_a_valid_secret_1337").is_ok())
    }
}
