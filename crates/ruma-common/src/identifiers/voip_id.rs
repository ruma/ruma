//! VoIP identifier.

use ruma_macros::IdZst;

/// A VoIP identifier.
///
/// VoIP IDs in Matrix are opaque strings. This type is provided simply for its semantic
/// value.
///
/// You can create one from a string (using `VoipId::parse()`) but the recommended way is to
/// use `VoipId::new()` to generate a random one. If that function is not available for you,
/// you need to activate this crate's `rand` Cargo feature.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdZst)]
pub struct VoipId(str);

impl VoipId {
    /// Creates a random VoIP identifier.
    ///
    /// This will currently be a UUID without hyphens, but no guarantees are made about the
    /// structure of client secrets generated from this function.
    #[cfg(feature = "rand")]
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> OwnedVoipId {
        let id = uuid::Uuid::new_v4();
        VoipId::from_borrowed(&id.simple().to_string()).to_owned()
    }
}
