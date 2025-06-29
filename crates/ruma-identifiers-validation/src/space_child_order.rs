use crate::Error;

/// Validate the `order` of an [`m.space.child`] event.
///
/// According to the specification, the order:
///
/// > Must consist of ASCII characters within the range `\x20` (space) and `\x7E` (~),
/// > inclusive. Must not exceed 50 characters.
///
/// Returns `Ok(())` if the order passes validation, or an error if the order doesn't respect
/// the rules from the spec, as it cannot be used for ordering.
///
/// [`m.space.child`]: https://spec.matrix.org/latest/client-server-api/#mspacechild
pub fn validate(s: &str) -> Result<(), Error> {
    if s.len() > 50 {
        return Err(Error::MaximumLengthExceeded);
    }

    if !s.bytes().all(|byte| (b'\x20'..=b'\x7E').contains(&byte)) {
        return Err(Error::InvalidCharacters);
    }

    Ok(())
}
