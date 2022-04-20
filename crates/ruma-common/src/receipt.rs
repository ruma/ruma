//! Common types for receipts.

use crate::{
    serde::{OrdAsRefStr, PartialEqAsRefStr, PartialOrdAsRefStr, StringEnum},
    PrivOwnedStr,
};

/// The type of receipt.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Debug, PartialOrdAsRefStr, OrdAsRefStr, PartialEqAsRefStr, Eq, StringEnum)]
#[non_exhaustive]
pub enum ReceiptType {
    /// m.read
    #[ruma_enum(rename = "m.read")]
    Read,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl ReceiptType {
    /// Creates a string slice from this `ReceiptType`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}
