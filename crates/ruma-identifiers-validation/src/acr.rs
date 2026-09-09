use crate::Error;

/// Validate the format of an Authentication Context Class Reference, as defined
/// in the [OpenID Connect specification].
///
/// The specification does not define an allowable character set for ACR values.
/// Because the `acr_values` query parameter accepts them as a space-separated
/// list, this function validates that the ACR is not empty and contains no whitespace.
///
/// [OpenID Connect specification]: https://openid.net/specs/openid-connect-core-1_0.html#IDToken:~:text=string%2E-,acr
pub fn validate(acr: &str) -> Result<(), Error> {
    if acr.is_empty() {
        return Err(Error::Empty);
    }

    if acr.chars().any(|char| char.is_ascii_whitespace()) {
        return Err(Error::InvalidCharacters);
    }

    Ok(())
}
