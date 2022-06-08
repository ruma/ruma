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

impl SessionId {
    #[doc(hidden)]
    pub const fn _priv_const_new(s: &str) -> Result<&Self, &'static str> {
        match validate_session_id(s) {
            Ok(()) => Ok(Self::from_borrowed(s)),
            Err(IdParseError::MaximumLengthExceeded) => {
                Err("Invalid Session ID: exceeds 255 bytes")
            }
            Err(IdParseError::InvalidCharacters) => {
                Err("Invalid Session ID: contains invalid characters")
            }
            Err(IdParseError::Empty) => Err("Invalid Session ID: empty"),
            Err(_) => unreachable!(),
        }
    }
}

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
