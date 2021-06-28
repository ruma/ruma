//! Common types for authentication.

use ruma_serde::StringEnum;

/// Access token types.
///
/// This type can hold an arbitrary string. To check for formats that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[non_exhaustive]
pub enum TokenType {
    /// Bearer token type
    Bearer,

    #[doc(hidden)]
    _Custom(String),
}

impl TokenType {
    /// Creates a string slice from this `TokenType`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}
