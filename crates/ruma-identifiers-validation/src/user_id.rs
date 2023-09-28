use crate::{parse_id, Error};

pub fn validate(s: &str) -> Result<(), Error> {
    let colon_idx = parse_id(s, b'@')?;
    let localpart = &s[1..colon_idx];
    let _ = localpart_is_fully_conforming(localpart)?;

    Ok(())
}

/// Check whether the given user id localpart is valid and fully conforming
///
/// Returns an `Err` for invalid user ID localparts, `Ok(false)` for historical user ID localparts
/// and `Ok(true)` for fully conforming user ID localparts.
///
/// With the `compat` feature enabled, this will also return `Ok(false)` for invalid user ID
/// localparts. User IDs that don't even meet the historical user ID restrictions exist in the wild
/// due to Synapse allowing them over federation. This will likely be fixed in an upcoming room
/// version; see [MSC2828](https://github.com/matrix-org/matrix-spec-proposals/pull/2828).
pub fn localpart_is_fully_conforming(localpart: &str) -> Result<bool, Error> {
    // See https://spec.matrix.org/latest/appendices/#user-identifiers
    let is_fully_conforming = !localpart.is_empty()
        && localpart.bytes().all(
            |b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'-' | b'.' | b'=' | b'_' | b'/' | b'+'),
        );

    if !is_fully_conforming {
        // If it's not fully conforming, check if it contains characters that are also disallowed
        // for historical user IDs, or is empty. If that's the case, return an error.
        // See https://spec.matrix.org/latest/appendices/#historical-user-ids
        #[cfg(not(feature = "compat-user-id"))]
        let is_invalid =
            localpart.is_empty() || localpart.bytes().any(|b| b < 0x21 || b == b':' || b > 0x7E);

        // In compat mode, allow anything except `:` to match Synapse. The `:` check is only needed
        // because this function can be called through `UserId::parse_with_servername`, otherwise
        // it would be impossible for the input to contain a `:`.
        #[cfg(feature = "compat-user-id")]
        let is_invalid = localpart.as_bytes().contains(&b':');

        if is_invalid {
            return Err(Error::InvalidCharacters);
        }
    }

    Ok(is_fully_conforming)
}
