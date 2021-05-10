//! Common types for receipts.

use ruma_serde::StringEnum;

/// The type of receipt.
#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq, StringEnum)]
pub enum ReceiptType {
    /// m.read
    #[ruma_enum(rename = "m.read")]
    Read,

    #[doc(hidden)]
    _Custom(String),
}
