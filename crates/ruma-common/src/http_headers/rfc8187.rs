//! Encoding and decoding functions according to [RFC 8187].
//!
//! [RFC 8187]: https://datatracker.ietf.org/doc/html/rfc8187

use std::borrow::Cow;

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};

/// The characters to percent-encode according to the `attr-char` set.
const ATTR_CHAR: AsciiSet = NON_ALPHANUMERIC
    .remove(b'!')
    .remove(b'#')
    .remove(b'$')
    .remove(b'&')
    .remove(b'+')
    .remove(b'-')
    .remove(b'.')
    .remove(b'^')
    .remove(b'_')
    .remove(b'`')
    .remove(b'|')
    .remove(b'~');

/// Encode the given string according to [RFC 8187].
///
/// [RFC 8187]: https://datatracker.ietf.org/doc/html/rfc8187
pub(super) fn encode(s: &str) -> String {
    let encoded = percent_encoding::utf8_percent_encode(s, &ATTR_CHAR);
    format!("utf-8''{encoded}")
}

/// Decode the given bytes according to [RFC 8187].
///
/// Only the UTF-8 character set is supported, all other character sets return an error.
///
/// [RFC 8187]: https://datatracker.ietf.org/doc/html/rfc8187
pub(super) fn decode(bytes: &[u8]) -> Result<Cow<'_, str>, Rfc8187DecodeError> {
    if bytes.is_empty() {
        return Err(Rfc8187DecodeError::Empty);
    }

    let mut parts = bytes.split(|b| *b == b'\'');
    let charset = parts.next().ok_or(Rfc8187DecodeError::WrongPartsCount)?;
    let _lang = parts.next().ok_or(Rfc8187DecodeError::WrongPartsCount)?;
    let encoded = parts.next().ok_or(Rfc8187DecodeError::WrongPartsCount)?;

    if parts.next().is_some() {
        return Err(Rfc8187DecodeError::WrongPartsCount);
    }

    if !charset.eq_ignore_ascii_case(b"utf-8") {
        return Err(Rfc8187DecodeError::NotUtf8);
    }

    // For maximum compatibility, do a lossy conversion.
    Ok(percent_encoding::percent_decode(encoded).decode_utf8_lossy())
}

/// All errors encountered when trying to decode a string according to [RFC 8187].
///
/// [RFC 8187]: https://datatracker.ietf.org/doc/html/rfc8187
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub(super) enum Rfc8187DecodeError {
    /// The string is empty.
    #[error("string is empty")]
    Empty,

    /// The string does not contain the right number of parts.
    #[error("string does not contain the right number of parts")]
    WrongPartsCount,

    /// The character set is not UTF-8.
    #[error("character set is not UTF-8")]
    NotUtf8,
}
