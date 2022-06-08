//! Matrix session ID.

use ruma_macros::IdZst;

use super::IdParseError;

/// A session ID.
///
/// Session IDs in Matrix are opaque character sequences of `[0-9a-zA-Z.=_-]`. Their length must
/// must not exceed 255 characters.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdZst)]
#[ruma_id(validate = validate_session_id)]
pub struct SessionId(str);

const fn validate_session_id(s: &str) -> Result<(), IdParseError> {
    if s.len() > 255 {
        return Err(IdParseError::MaximumLengthExceeded);
    } else if contains_invalid_byte(s.as_bytes()) {
        return Err(IdParseError::InvalidCharacters);
    } else if s.is_empty() {
        return Err(IdParseError::Empty);
    }

    Ok(())
}

const fn contains_invalid_byte(mut bytes: &[u8]) -> bool {
    // non-const form:
    //
    // bytes.iter().all(|b| b.is_ascii_alphanumeric() || b".=_-".contains(&b))
    loop {
        if let Some((byte, rest)) = bytes.split_first() {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'=' | b'_' | b'-') {
                bytes = rest;
            } else {
                break true;
            }
        } else {
            break false;
        }
    }
}
