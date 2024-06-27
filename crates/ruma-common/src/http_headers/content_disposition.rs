//! Types to (de)serialize the `Content-Disposition` HTTP header.

use std::{fmt, ops::Deref, str::FromStr};

use ruma_macros::{
    AsRefStr, AsStrAsRefStr, DebugAsRefStr, DisplayAsRefStr, OrdAsRefStr, PartialOrdAsRefStr,
};

use super::{
    is_tchar, is_token, quote_ascii_string_if_required, rfc8187, sanitize_for_ascii_quoted_string,
    unescape_string,
};

/// The value of a `Content-Disposition` HTTP header.
///
/// This implementation supports the `Content-Disposition` header format as defined for HTTP in [RFC
/// 6266].
///
/// The only supported parameter is `filename`. It is encoded or decoded as needed, using a quoted
/// string or the `ext-token = ext-value` format, with the encoding defined in [RFC 8187].
///
/// This implementation does not support serializing to the format defined for the
/// `multipart/form-data` content type in [RFC 7578]. It should however manage to parse the
/// disposition type and filename parameter of the body parts.
///
/// [RFC 6266]: https://datatracker.ietf.org/doc/html/rfc6266
/// [RFC 8187]: https://datatracker.ietf.org/doc/html/rfc8187
/// [RFC 7578]: https://datatracker.ietf.org/doc/html/rfc7578
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ContentDisposition {
    /// The disposition type.
    pub disposition_type: ContentDispositionType,

    /// The filename of the content.
    pub filename: Option<String>,
}

impl ContentDisposition {
    /// Creates a new `ContentDisposition` with the given disposition type.
    pub fn new(disposition_type: ContentDispositionType) -> Self {
        Self { disposition_type, filename: None }
    }

    /// Add the given filename to this `ContentDisposition`.
    pub fn with_filename(mut self, filename: Option<String>) -> Self {
        self.filename = filename;
        self
    }
}

impl fmt::Display for ContentDisposition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.disposition_type)?;

        if let Some(filename) = &self.filename {
            if filename.is_ascii() {
                // First, remove all non-quotable characters, that is control characters.
                let filename = sanitize_for_ascii_quoted_string(filename);

                // We can use the filename parameter.
                write!(f, "; filename={}", quote_ascii_string_if_required(&filename))?;
            } else {
                // We need to use RFC 8187 encoding.
                write!(f, "; filename*={}", rfc8187::encode(filename))?;
            }
        }

        Ok(())
    }
}

impl TryFrom<&[u8]> for ContentDisposition {
    type Error = ContentDispositionParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut pos = 0;

        skip_ascii_whitespaces(value, &mut pos);

        if pos == value.len() {
            return Err(ContentDispositionParseError::MissingDispositionType);
        }

        let disposition_type_start = pos;

        // Find the next whitespace or `;`.
        while let Some(byte) = value.get(pos) {
            if byte.is_ascii_whitespace() || *byte == b';' {
                break;
            }

            pos += 1;
        }

        let disposition_type =
            ContentDispositionType::try_from(&value[disposition_type_start..pos])?;

        // The `filename*` parameter (`filename_ext` here) using UTF-8 encoding should be used, but
        // it is likely to be after the `filename` parameter containing only ASCII
        // characters if both are present.
        let mut filename_ext = None;
        let mut filename = None;

        // Parse the parameters. We ignore parameters that fail to parse for maximum compatibility.
        while pos != value.len() {
            if let Some(param) = RawParam::parse_next(value, &mut pos) {
                if param.name.eq_ignore_ascii_case(b"filename*") {
                    if let Some(value) = param.decode_value() {
                        filename_ext = Some(value);
                        // We can stop parsing, this is the only parameter that we need.
                        break;
                    }
                } else if param.name.eq_ignore_ascii_case(b"filename") {
                    if let Some(value) = param.decode_value() {
                        filename = Some(value);
                    }
                }
            }
        }

        Ok(Self { disposition_type, filename: filename_ext.or(filename) })
    }
}

impl FromStr for ContentDisposition {
    type Err = ContentDispositionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.as_bytes().try_into()
    }
}

/// A raw parameter in a `Content-Disposition` HTTP header.
struct RawParam<'a> {
    name: &'a [u8],
    value: &'a [u8],
    is_quoted_string: bool,
}

impl<'a> RawParam<'a> {
    /// Parse the next `RawParam` in the given bytes, starting at the given position.
    ///
    /// The position is updated during the parsing.
    ///
    /// Returns `None` if no parameter was found or if an error occurred when parsing the
    /// parameter.
    fn parse_next(bytes: &'a [u8], pos: &mut usize) -> Option<Self> {
        let name = parse_param_name(bytes, pos)?;

        skip_ascii_whitespaces(bytes, pos);

        if *pos == bytes.len() {
            // We are at the end of the bytes and only have the parameter name.
            return None;
        }
        if bytes[*pos] != b'=' {
            // We should have an equal sign, there is a problem with the bytes and we can't recover
            // from it.
            // Skip to the end to stop the parsing.
            *pos = bytes.len();
            return None;
        }

        // Skip the equal sign.
        *pos += 1;

        skip_ascii_whitespaces(bytes, pos);

        let (value, is_quoted_string) = parse_param_value(bytes, pos)?;

        Some(Self { name, value, is_quoted_string })
    }

    /// Decode the value of this `RawParam`.
    ///
    /// Returns `None` if decoding the param failed.
    fn decode_value(&self) -> Option<String> {
        if self.name.ends_with(b"*") {
            rfc8187::decode(self.value).ok().map(|s| s.into_owned())
        } else {
            let s = String::from_utf8_lossy(self.value);

            if self.is_quoted_string {
                Some(unescape_string(&s))
            } else {
                Some(s.into_owned())
            }
        }
    }
}

/// Skip ASCII whitespaces in the given bytes, starting at the given position.
///
/// The position is updated to after the whitespaces.
fn skip_ascii_whitespaces(bytes: &[u8], pos: &mut usize) {
    while let Some(byte) = bytes.get(*pos) {
        if !byte.is_ascii_whitespace() {
            break;
        }

        *pos += 1;
    }
}

/// Parse a parameter name in the given bytes, starting at the given position.
///
/// The position is updated while parsing.
///
/// Returns `None` if the end of the bytes was reached, or if an error was encountered.
fn parse_param_name<'a>(bytes: &'a [u8], pos: &mut usize) -> Option<&'a [u8]> {
    skip_ascii_whitespaces(bytes, pos);

    if *pos == bytes.len() {
        // We are at the end of the bytes and didn't find anything.
        return None;
    }

    let name_start = *pos;

    // Find the end of the parameter name. The name can only contain token chars.
    while let Some(byte) = bytes.get(*pos) {
        if !is_tchar(*byte) {
            break;
        }

        *pos += 1;
    }

    if *pos == bytes.len() {
        // We are at the end of the bytes and only have the parameter name.
        return None;
    }
    if bytes[*pos] == b';' {
        // We are at the end of the parameter and only have the parameter name, skip the `;` and
        // parse the next parameter.
        *pos += 1;
        return None;
    }

    let name = &bytes[name_start..*pos];

    if name.is_empty() {
        // It's probably a syntax error, we cannot recover from it.
        *pos = bytes.len();
        return None;
    }

    Some(name)
}

/// Parse a parameter value in the given bytes, starting at the given position.
///
/// The position is updated while parsing.
///
/// Returns a `(value, is_quoted_string)` tuple if parsing succeeded.
/// Returns `None` if the end of the bytes was reached, or if an error was encountered.
fn parse_param_value<'a>(bytes: &'a [u8], pos: &mut usize) -> Option<(&'a [u8], bool)> {
    skip_ascii_whitespaces(bytes, pos);

    if *pos == bytes.len() {
        // We are at the end of the bytes and didn't find anything.
        return None;
    }

    let is_quoted_string = bytes[*pos] == b'"';
    if is_quoted_string {
        // Skip the start double quote.
        *pos += 1;
    }

    let value_start = *pos;

    // Keep track of whether the next byte is escaped with a backslash.
    let mut escape_next = false;

    // Find the end of the value, it's a whitespace or a semi-colon, or a double quote if the string
    // is quoted.
    while let Some(byte) = bytes.get(*pos) {
        if !is_quoted_string && (byte.is_ascii_whitespace() || *byte == b';') {
            break;
        }

        if is_quoted_string && *byte == b'"' && !escape_next {
            break;
        }

        escape_next = *byte == b'\\' && !escape_next;

        *pos += 1;
    }

    let value = &bytes[value_start..*pos];

    if is_quoted_string && *pos != bytes.len() {
        // Skip the end double quote.
        *pos += 1;
    }

    skip_ascii_whitespaces(bytes, pos);

    // Check for parameters separator if we are not at the end of the string.
    if *pos != bytes.len() {
        if bytes[*pos] == b';' {
            // Skip the `;` at the end of the parameter.
            *pos += 1;
        } else {
            // We should have a `;`, there is a problem with the bytes and we can't recover
            // from it.
            // Skip to the end to stop the parsing.
            *pos = bytes.len();
            return None;
        }
    }

    Some((value, is_quoted_string))
}

/// An error encountered when trying to parse an invalid [`ContentDisposition`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ContentDispositionParseError {
    /// The disposition type is missing.
    #[error("disposition type is missing")]
    MissingDispositionType,

    /// The disposition type is invalid.
    #[error("invalid disposition type: {0}")]
    InvalidDispositionType(#[from] TokenStringParseError),
}

/// A disposition type in the `Content-Disposition` HTTP header as defined in [Section 4.2 of RFC
/// 6266].
///
/// This type can hold an arbitrary [`TokenString`]. To build this with a custom value, convert it
/// from a `TokenString` with `::from()` / `.into()`. To check for values that are not available as
/// a documented variant here, use its string representation, obtained through
/// [`.as_str()`](Self::as_str()).
///
/// Comparisons with other string types are done case-insensitively.
///
/// [Section 4.2 of RFC 6266]: https://datatracker.ietf.org/doc/html/rfc6266#section-4.2
#[derive(
    Clone,
    Default,
    AsRefStr,
    DebugAsRefStr,
    AsStrAsRefStr,
    DisplayAsRefStr,
    PartialOrdAsRefStr,
    OrdAsRefStr,
)]
#[ruma_enum(rename_all = "lowercase")]
#[non_exhaustive]
pub enum ContentDispositionType {
    /// The content can be displayed.
    ///
    /// This is the default.
    #[default]
    Inline,

    /// The content should be downloaded instead of displayed.
    Attachment,

    #[doc(hidden)]
    _Custom(TokenString),
}

impl ContentDispositionType {
    /// Try parsing a `&str` into a `ContentDispositionType`.
    pub fn parse(s: &str) -> Result<Self, TokenStringParseError> {
        Self::from_str(s)
    }
}

impl From<TokenString> for ContentDispositionType {
    fn from(value: TokenString) -> Self {
        if value.eq_ignore_ascii_case("inline") {
            Self::Inline
        } else if value.eq_ignore_ascii_case("attachment") {
            Self::Attachment
        } else {
            Self::_Custom(value)
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for ContentDispositionType {
    type Error = TokenStringParseError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.eq_ignore_ascii_case(b"inline") {
            Ok(Self::Inline)
        } else if value.eq_ignore_ascii_case(b"attachment") {
            Ok(Self::Attachment)
        } else {
            TokenString::try_from(value).map(Self::_Custom)
        }
    }
}

impl FromStr for ContentDispositionType {
    type Err = TokenStringParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.as_bytes().try_into()
    }
}

impl PartialEq<ContentDispositionType> for ContentDispositionType {
    fn eq(&self, other: &ContentDispositionType) -> bool {
        self.as_str().eq_ignore_ascii_case(other.as_str())
    }
}

impl Eq for ContentDispositionType {}

impl PartialEq<TokenString> for ContentDispositionType {
    fn eq(&self, other: &TokenString) -> bool {
        self.as_str().eq_ignore_ascii_case(other.as_str())
    }
}

impl<'a> PartialEq<&'a str> for ContentDispositionType {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_str().eq_ignore_ascii_case(other)
    }
}

/// A non-empty string consisting only of `token`s as defined in [RFC 9110 Section 3.2.6].
///
/// This is a string that can only contain a limited character set.
///
/// [RFC 7230 Section 3.2.6]: https://datatracker.ietf.org/doc/html/rfc7230#section-3.2.6
#[derive(
    Clone,
    PartialEq,
    Eq,
    DebugAsRefStr,
    AsStrAsRefStr,
    DisplayAsRefStr,
    PartialOrdAsRefStr,
    OrdAsRefStr,
)]
pub struct TokenString(Box<str>);

impl TokenString {
    /// Try parsing a `&str` into a `TokenString`.
    pub fn parse(s: &str) -> Result<Self, TokenStringParseError> {
        Self::from_str(s)
    }
}

impl Deref for TokenString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl AsRef<str> for TokenString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'a> PartialEq<&'a str> for TokenString {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_str().eq(*other)
    }
}

impl<'a> TryFrom<&'a [u8]> for TokenString {
    type Error = TokenStringParseError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(TokenStringParseError::Empty)
        } else if is_token(value) {
            let s = std::str::from_utf8(value).expect("ASCII bytes are valid UTF-8");
            Ok(Self(s.into()))
        } else {
            Err(TokenStringParseError::InvalidCharacter)
        }
    }
}

impl FromStr for TokenString {
    type Err = TokenStringParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.as_bytes().try_into()
    }
}

/// The parsed string contains a character not allowed for a [`TokenString`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum TokenStringParseError {
    /// The string is empty.
    #[error("string is empty")]
    Empty,

    /// The string contains an invalid character for a token string.
    #[error("string contains invalid character")]
    InvalidCharacter,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{ContentDisposition, ContentDispositionType};

    #[test]
    fn parse_content_disposition_valid() {
        // Only disposition type.
        let content_disposition = ContentDisposition::from_str("inline").unwrap();
        assert_eq!(content_disposition.disposition_type, ContentDispositionType::Inline);
        assert_eq!(content_disposition.filename, None);

        // Only disposition type with separator.
        let content_disposition = ContentDisposition::from_str("attachment;").unwrap();
        assert_eq!(content_disposition.disposition_type, ContentDispositionType::Attachment);
        assert_eq!(content_disposition.filename, None);

        // Unknown disposition type and parameters.
        let content_disposition =
            ContentDisposition::from_str("custom; foo=bar; foo*=utf-8''b%C3%A0r'").unwrap();
        assert_eq!(content_disposition.disposition_type.as_str(), "custom");
        assert_eq!(content_disposition.filename, None);

        // Disposition type and filename.
        let content_disposition = ContentDisposition::from_str("inline; filename=my_file").unwrap();
        assert_eq!(content_disposition.disposition_type, ContentDispositionType::Inline);
        assert_eq!(content_disposition.filename.unwrap(), "my_file");

        // Case insensitive.
        let content_disposition = ContentDisposition::from_str("INLINE; FILENAME=my_file").unwrap();
        assert_eq!(content_disposition.disposition_type, ContentDispositionType::Inline);
        assert_eq!(content_disposition.filename.unwrap(), "my_file");

        // Extra spaces.
        let content_disposition =
            ContentDisposition::from_str("  INLINE   ;FILENAME =   my_file   ").unwrap();
        assert_eq!(content_disposition.disposition_type, ContentDispositionType::Inline);
        assert_eq!(content_disposition.filename.unwrap(), "my_file");

        // Unsupported filename* is skipped and falls back to ASCII filename.
        let content_disposition = ContentDisposition::from_str(
            r#"attachment; filename*=iso-8859-1''foo-%E4.html; filename="foo-a.html"#,
        )
        .unwrap();
        assert_eq!(content_disposition.disposition_type, ContentDispositionType::Attachment);
        assert_eq!(content_disposition.filename.unwrap(), "foo-a.html");

        // filename could be UTF-8 for extra compatibility (with `form-data` for example).
        let content_disposition =
            ContentDisposition::from_str(r#"form-data; name=upload; filename="文件.webp""#)
                .unwrap();
        assert_eq!(content_disposition.disposition_type.as_str(), "form-data");
        assert_eq!(content_disposition.filename.unwrap(), "文件.webp");
    }

    #[test]
    fn parse_content_disposition_invalid_type() {
        // Empty.
        ContentDisposition::from_str("").unwrap_err();

        // Missing disposition type.
        ContentDisposition::from_str("; foo=bar").unwrap_err();
    }

    #[test]
    fn parse_content_disposition_invalid_parameters() {
        // Unexpected `:` after parameter name, filename parameter is not reached.
        let content_disposition =
            ContentDisposition::from_str("inline; foo:bar; filename=my_file").unwrap();
        assert_eq!(content_disposition.disposition_type, ContentDispositionType::Inline);
        assert_eq!(content_disposition.filename, None);

        // Same error, but after filename, so filename was parser.
        let content_disposition =
            ContentDisposition::from_str("inline; filename=my_file; foo:bar").unwrap();
        assert_eq!(content_disposition.disposition_type, ContentDispositionType::Inline);
        assert_eq!(content_disposition.filename.unwrap(), "my_file");

        // Missing `;` between parameters, filename parameter is not parsed successfully.
        let content_disposition =
            ContentDisposition::from_str("inline; filename=my_file foo=bar").unwrap();
        assert_eq!(content_disposition.disposition_type, ContentDispositionType::Inline);
        assert_eq!(content_disposition.filename, None);
    }

    #[test]
    fn content_disposition_serialize() {
        // Only disposition type.
        let content_disposition = ContentDisposition::new(ContentDispositionType::Inline);
        let serialized = content_disposition.to_string();
        assert_eq!(serialized, "inline");

        // Disposition type and ASCII filename without space.
        let content_disposition = ContentDisposition::new(ContentDispositionType::Attachment)
            .with_filename(Some("my_file".to_owned()));
        let serialized = content_disposition.to_string();
        assert_eq!(serialized, "attachment; filename=my_file");

        // Disposition type and ASCII filename with space.
        let content_disposition = ContentDisposition::new(ContentDispositionType::Attachment)
            .with_filename(Some("my file".to_owned()));
        let serialized = content_disposition.to_string();
        assert_eq!(serialized, r#"attachment; filename="my file""#);

        // Disposition type and ASCII filename with double quote and backslash.
        let content_disposition = ContentDisposition::new(ContentDispositionType::Attachment)
            .with_filename(Some(r#""my"\file"#.to_owned()));
        let serialized = content_disposition.to_string();
        assert_eq!(serialized, r#"attachment; filename="\"my\"\\file""#);

        // Disposition type and UTF-8 filename.
        let content_disposition = ContentDisposition::new(ContentDispositionType::Attachment)
            .with_filename(Some("Mi Corazón".to_owned()));
        let serialized = content_disposition.to_string();
        assert_eq!(serialized, "attachment; filename*=utf-8''Mi%20Coraz%C3%B3n");

        // Sanitized filename.
        let content_disposition = ContentDisposition::new(ContentDispositionType::Attachment)
            .with_filename(Some("my\r\nfile".to_owned()));
        let serialized = content_disposition.to_string();
        assert_eq!(serialized, "attachment; filename=myfile");
    }

    #[test]
    fn rfc6266_examples() {
        // Basic syntax with unquoted filename.
        let unquoted = "Attachment; filename=example.html";
        let content_disposition = ContentDisposition::from_str(unquoted).unwrap();

        assert_eq!(content_disposition.disposition_type, ContentDispositionType::Attachment);
        assert_eq!(content_disposition.filename.as_deref().unwrap(), "example.html");

        let reserialized = content_disposition.to_string();
        assert_eq!(reserialized, "attachment; filename=example.html");

        // With quoted filename, case insensitivity and extra whitespaces.
        let quoted = r#"INLINE; FILENAME= "an example.html""#;
        let content_disposition = ContentDisposition::from_str(quoted).unwrap();

        assert_eq!(content_disposition.disposition_type, ContentDispositionType::Inline);
        assert_eq!(content_disposition.filename.as_deref().unwrap(), "an example.html");

        let reserialized = content_disposition.to_string();
        assert_eq!(reserialized, r#"inline; filename="an example.html""#);

        // With RFC 8187-encoded UTF-8 filename.
        let rfc8187 = "attachment; filename*= UTF-8''%e2%82%ac%20rates";
        let content_disposition = ContentDisposition::from_str(rfc8187).unwrap();

        assert_eq!(content_disposition.disposition_type, ContentDispositionType::Attachment);
        assert_eq!(content_disposition.filename.as_deref().unwrap(), "€ rates");

        let reserialized = content_disposition.to_string();
        assert_eq!(reserialized, r#"attachment; filename*=utf-8''%E2%82%AC%20rates"#);

        // With RFC 8187-encoded UTF-8 filename with fallback ASCII filename.
        let rfc8187_with_fallback =
            r#"attachment; filename="EURO rates"; filename*=utf-8''%e2%82%ac%20rates"#;
        let content_disposition = ContentDisposition::from_str(rfc8187_with_fallback).unwrap();

        assert_eq!(content_disposition.disposition_type, ContentDispositionType::Attachment);
        assert_eq!(content_disposition.filename.as_deref().unwrap(), "€ rates");
    }

    #[test]
    fn rfc8187_examples() {
        // Those examples originate from RFC 8187, but are changed to fit the expectations here:
        //
        // - A disposition type is added
        // - The title parameter is renamed to filename

        // Basic syntax with unquoted filename.
        let unquoted = "attachment; foo= bar; filename=Economy";
        let content_disposition = ContentDisposition::from_str(unquoted).unwrap();

        assert_eq!(content_disposition.disposition_type, ContentDispositionType::Attachment);
        assert_eq!(content_disposition.filename.as_deref().unwrap(), "Economy");

        let reserialized = content_disposition.to_string();
        assert_eq!(reserialized, "attachment; filename=Economy");

        // With quoted filename.
        let quoted = r#"attachment; foo=bar; filename="US-$ rates""#;
        let content_disposition = ContentDisposition::from_str(quoted).unwrap();

        assert_eq!(content_disposition.disposition_type, ContentDispositionType::Attachment);
        assert_eq!(content_disposition.filename.as_deref().unwrap(), "US-$ rates");

        let reserialized = content_disposition.to_string();
        assert_eq!(reserialized, r#"attachment; filename="US-$ rates""#);

        // With RFC 8187-encoded UTF-8 filename.
        let rfc8187 = "attachment; foo=bar; filename*=utf-8'en'%C2%A3%20rates";
        let content_disposition = ContentDisposition::from_str(rfc8187).unwrap();

        assert_eq!(content_disposition.disposition_type, ContentDispositionType::Attachment);
        assert_eq!(content_disposition.filename.as_deref().unwrap(), "£ rates");

        let reserialized = content_disposition.to_string();
        assert_eq!(reserialized, r#"attachment; filename*=utf-8''%C2%A3%20rates"#);

        // With RFC 8187-encoded UTF-8 filename again.
        let rfc8187_other =
            r#"attachment; foo=bar; filename*=UTF-8''%c2%a3%20and%20%e2%82%ac%20rates"#;
        let content_disposition = ContentDisposition::from_str(rfc8187_other).unwrap();

        assert_eq!(content_disposition.disposition_type, ContentDispositionType::Attachment);
        assert_eq!(content_disposition.filename.as_deref().unwrap(), "£ and € rates");

        let reserialized = content_disposition.to_string();
        assert_eq!(
            reserialized,
            r#"attachment; filename*=utf-8''%C2%A3%20and%20%E2%82%AC%20rates"#
        );

        // With RFC 8187-encoded UTF-8 filename with fallback ASCII filename.
        let rfc8187_with_fallback = r#"attachment; foo=bar; filename="EURO exchange rates"; filename*=utf-8''%e2%82%ac%20exchange%20rates"#;
        let content_disposition = ContentDisposition::from_str(rfc8187_with_fallback).unwrap();

        assert_eq!(content_disposition.disposition_type, ContentDispositionType::Attachment);
        assert_eq!(content_disposition.filename.as_deref().unwrap(), "€ exchange rates");

        let reserialized = content_disposition.to_string();
        assert_eq!(reserialized, r#"attachment; filename*=utf-8''%E2%82%AC%20exchange%20rates"#);
    }
}
