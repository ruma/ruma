//! Common types for authentication.

use crate::{serde::StringEnum, PrivOwnedStr};

/// Access token types.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[non_exhaustive]
pub enum TokenType {
    /// Bearer token type
    Bearer,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl TokenType {
    /// Creates a string slice from this `TokenType`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}
