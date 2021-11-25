use crate::{parse_id, Error};

pub fn validate(s: &str) -> Result<(), Error> {
    let colon_idx = parse_id(s, &['@'])?;
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
/// version; see <https://github.com/matrix-org/matrix-doc/pull/2828>.
pub fn localpart_is_fully_conforming(localpart: &str) -> Result<bool, Error> {
    // See https://matrix.org/docs/spec/appendices#user-identifiers
    let is_fully_conforming = localpart
        .bytes()
        .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'-' | b'.' | b'=' | b'_' | b'/'));

    // If it's not fully conforming, check if it contains characters that are also disallowed
    // for historical user IDs. If there are, return an error.
    // See https://matrix.org/docs/spec/appendices#historical-user-ids
    #[cfg(not(feature = "compat"))]
    if !is_fully_conforming && localpart.bytes().any(|b| b < 0x21 || b == b':' || b > 0x7E) {
        Err(Error::InvalidCharacters)
    } else {
        Ok(is_fully_conforming)
    }

    #[cfg(feature = "compat")]
    Ok(is_fully_conforming)
}
