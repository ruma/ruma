//! Authentication Context Class Reference.

use ruma_macros::IdDst;

/// An Authentication Context Class Reference, as defined in the
/// [OpenID Connect specification]. ACRs are opaque tokens containing
/// no whitespace.
///
/// [OpenID Connect specification]: https://openid.net/specs/openid-connect-core-1_0.html#IDToken:~:text=string%2E-,acr
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdDst)]
#[ruma_id(validate = ruma_identifiers_validation::acr::validate)]
pub struct Acr(str);

#[cfg(test)]
mod tests {
    use ruma_identifiers_validation::Error;

    use super::Acr;

    #[test]
    fn validate_acr() {
        Acr::parse("urn:example:cve").unwrap();
        Acr::parse("🛡️").unwrap();

        assert_eq!(Acr::parse("etaoin shrdlu"), Err(Error::InvalidCharacters));
        assert_eq!(Acr::parse(""), Err(Error::Empty));
    }
}
