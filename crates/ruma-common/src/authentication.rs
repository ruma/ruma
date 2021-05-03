//! Common types for authentication.

use ruma_serde::StringEnum;

/// Access token types.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
pub enum TokenType {
    /// Bearer token type
    Bearer,

    #[doc(hidden)]
    _Custom(String),
}
