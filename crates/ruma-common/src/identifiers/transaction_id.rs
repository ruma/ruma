/// A Matrix transaction ID.
///
/// Transaction IDs in Matrix are opaque strings. This type is provided simply for its semantic
/// value.
///
/// You can create one from a string (using `.into()`) but the recommended way is to use
/// `TransactionId::new()` to generate a random one. If that function is not available for you, you
/// need to activate this crate's `rand` Cargo feature.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TransactionId(str);

impl TransactionId {
    /// Creates a random transaction ID.
    ///
    /// This will currently be a UUID without hyphens, but no guarantees are made about the
    /// structure of transaction IDs generated from this function.
    #[cfg(feature = "rand")]
    pub fn new() -> Box<Self> {
        let id = uuid::Uuid::new_v4();
        Self::from_owned(id.to_simple().to_string().into_boxed_str())
    }
}

opaque_identifier!(TransactionId);
