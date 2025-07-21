use crate::{localpart_is_backwards_compatible, parse_id, Error, ID_MAX_BYTES};

/// Validate a [user ID] as used by clients.
///
/// [user ID]: https://spec.matrix.org/latest/appendices/#user-identifiers
pub fn validate(s: &str) -> Result<(), Error> {
    let colon_idx = parse_id(s, b'@')?;
    let localpart = &s[1..colon_idx];

    localpart_is_backwards_compatible(localpart)?;

    Ok(())
}

/// Validate a [user ID] to follow the spec recommendations when generating them.
///
/// [user ID]: https://spec.matrix.org/latest/appendices/#user-identifiers
pub fn validate_strict(s: &str) -> Result<(), Error> {
    // Since the length check can be disabled with `compat-arbitrary-length-ids`, check it again
    // here.
    if s.len() > ID_MAX_BYTES {
        return Err(Error::MaximumLengthExceeded);
    }

    let colon_idx = parse_id(s, b'@')?;
    let localpart = &s[1..colon_idx];

    if !localpart_is_fully_conforming(localpart)? {
        return Err(Error::InvalidCharacters);
    }

    Ok(())
}

/// Check whether the given [user ID] localpart is valid and fully conforming.
///
/// Returns an `Err` for invalid user ID localparts, `Ok(false)` for historical user ID localparts
/// and `Ok(true)` for fully conforming user ID localparts.
///
/// [user ID]: https://spec.matrix.org/latest/appendices/#user-identifiers
pub fn localpart_is_fully_conforming(localpart: &str) -> Result<bool, Error> {
    if localpart.is_empty() {
        return Err(Error::Empty);
    }

    // See https://spec.matrix.org/latest/appendices/#user-identifiers
    let is_fully_conforming = localpart
        .bytes()
        .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'-' | b'.' | b'=' | b'_' | b'/' | b'+'));

    if !is_fully_conforming {
        // If it's not fully conforming, check if it contains characters that are also disallowed
        // for historical user IDs, or is empty. If that's the case, return an error.
        // See https://spec.matrix.org/latest/appendices/#historical-user-ids
        let is_invalid_historical = localpart.bytes().any(|b| b < 0x21 || b == b':' || b > 0x7E);

        if is_invalid_historical {
            return Err(Error::InvalidCharacters);
        }
    }

    Ok(is_fully_conforming)
}
