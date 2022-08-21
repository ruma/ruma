//! Client secret identifier.

use ruma_macros::IdZst;

/// A client secret.
///
/// Client secrets in Matrix are opaque character sequences of `[0-9a-zA-Z.=_-]`. Their length must
/// must not exceed 255 characters.
///
/// You can create one from a string (using `ClientSecret::parse()`) but the recommended way is to
/// use `ClientSecret::new()` to generate a random one. If that function is not available for you,
/// you need to activate this crate's `rand` Cargo feature.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdZst)]
#[ruma_id(validate = ruma_identifiers_validation::client_secret::validate)]
pub struct ClientSecret(str);

impl ClientSecret {
    /// Creates a random client secret.
    ///
    /// This will currently be a UUID without hyphens, but no guarantees are made about the
    /// structure of client secrets generated from this function.
    #[cfg(feature = "rand")]
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> OwnedClientSecret {
        let id = uuid::Uuid::new_v4();
        ClientSecret::from_borrowed(&id.simple().to_string()).to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::ClientSecret;

    #[test]
    fn valid_secret() {
        <&ClientSecret>::try_from("this_=_a_valid_secret_1337").unwrap();
    }
}
