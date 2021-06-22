//! Client secret identifier.

use ruma_identifiers_validation::client_secret::validate;

opaque_identifier_validated! {
    /// A client secret.
    ///
    /// Client secrets in Matrix are opaque character sequences of `[0-9a-zA-Z.=_-]`. Their length must
    /// must not exceed 255 characters.
    pub type ClientSecret;
    validate;
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::ClientSecret;

    #[test]
    fn valid_secret() {
        assert!(<&ClientSecret>::try_from("this_=_a_valid_secret_1337").is_ok())
    }
}
