//! Helpers for HTTP headers.

use std::borrow::Cow;

mod content_disposition;
mod rfc8187;

pub use self::content_disposition::{
    ContentDisposition, ContentDispositionParseError, ContentDispositionType, TokenString,
    TokenStringParseError,
};

/// Whether the given byte is a [`token` char].
///
/// [`token` char]: https://datatracker.ietf.org/doc/html/rfc9110#section-5.6.2
pub const fn is_tchar(b: u8) -> bool {
    b.is_ascii_alphanumeric()
        || matches!(
            b,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}

/// Whether the given bytes slice is a [`token`].
///
/// [`token`]: https://datatracker.ietf.org/doc/html/rfc9110#section-5.6.2
pub fn is_token(bytes: &[u8]) -> bool {
    bytes.iter().all(|b| is_tchar(*b))
}

/// Whether the given string is a [`token`].
///
/// [`token`]: https://datatracker.ietf.org/doc/html/rfc9110#section-5.6.2
pub fn is_token_string(s: &str) -> bool {
    is_token(s.as_bytes())
}

/// Whether the given char is a [visible US-ASCII char].
///
/// [visible US-ASCII char]: https://datatracker.ietf.org/doc/html/rfc5234#appendix-B.1
pub const fn is_vchar(c: char) -> bool {
    matches!(c, '\x21'..='\x7E')
}

/// Whether the given char is in the US-ASCII character set and allowed inside a [quoted string].
///
/// Contrary to the definition of quoted strings, this doesn't allow `obs-text` characters, i.e.
/// non-US-ASCII characters, as we usually deal with UTF-8 strings rather than ISO-8859-1 strings.
///
/// [quoted string]: https://datatracker.ietf.org/doc/html/rfc9110#section-5.6.4
pub const fn is_ascii_string_quotable(c: char) -> bool {
    is_vchar(c) || matches!(c, '\x09' | '\x20')
}

/// Remove characters that do not pass [`is_ascii_string_quotable()`] from the given string.
///
/// [quoted string]: https://datatracker.ietf.org/doc/html/rfc9110#section-5.6.4
pub fn sanitize_for_ascii_quoted_string(value: &str) -> Cow<'_, str> {
    if value.chars().all(is_ascii_string_quotable) {
        return Cow::Borrowed(value);
    }

    Cow::Owned(value.chars().filter(|c| is_ascii_string_quotable(*c)).collect())
}

/// If the US-ASCII field value does not contain only token chars, convert it to a [quoted string].
///
/// The string should be sanitized with [`sanitize_for_ascii_quoted_string()`] or should only
/// contain characters that pass [`is_ascii_string_quotable()`].
///
/// [quoted string]: https://datatracker.ietf.org/doc/html/rfc9110#section-5.6.4
pub fn quote_ascii_string_if_required(value: &str) -> Cow<'_, str> {
    if !value.is_empty() && is_token_string(value) {
        return Cow::Borrowed(value);
    }

    let value = value.replace('\\', r#"\\"#).replace('"', r#"\""#);
    Cow::Owned(format!("\"{value}\""))
}

/// Removes the escape backslashes in the given string.
pub fn unescape_string(s: &str) -> String {
    let mut is_escaped = false;

    s.chars()
        .filter(|c| {
            is_escaped = *c == '\\' && !is_escaped;
            !is_escaped
        })
        .collect()
}
