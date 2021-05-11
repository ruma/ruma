//! Common types for receipts.

use ruma_serde::{OrdAsRefStr, PartialEqAsRefStr, PartialOrdAsRefStr, StringEnum};

/// The type of receipt.
#[derive(Clone, Debug, PartialOrdAsRefStr, OrdAsRefStr, PartialEqAsRefStr, Eq, StringEnum)]
pub enum ReceiptType {
    /// m.read
    #[ruma_enum(rename = "m.read")]
    Read,

    #[doc(hidden)]
    _Custom(String),
}
