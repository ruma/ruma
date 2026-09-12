use crate::Error;

/// Validate the format of an OAuth scope, as defined in [RFC6749].
///
/// OAuth scopes may only contain:
/// - \x21 (`!`)
/// - Characters in the range \x23 (`#`) to \x5B (`[`)
/// - Characters in the range \x5D (`]`) to \x7E (`~`)
///
/// [RFC6749]: https://datatracker.ietf.org/doc/html/rfc6749#section-3.3
pub fn validate(scope: &str) -> Result<(), Error> {
    if scope.is_empty() {
        return Err(Error::Empty);
    }

    if !scope.chars().all(|char| matches!(char, '\x21' | '\x23'..='\x5B' | '\x5D'..='\x7E')) {
        return Err(Error::InvalidCharacters);
    }

    Ok(())
}
