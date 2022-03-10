//! Common types for receipts.

use crate::{
    serde::{OrdAsRefStr, PartialEqAsRefStr, PartialOrdAsRefStr, StringEnum},
    PrivOwnedStr,
};

/// The type of receipt.
///
/// This type can hold an arbitrary string. To check for formats that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
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
