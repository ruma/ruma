//! VoIP identifier.

use ruma_macros::ruma_id;

/// A VoIP identifier.
///
/// VoIP IDs in Matrix are opaque strings. This type is provided simply for its semantic
/// value.
///
/// You can create one from a string (using `VoipId::from()`) but the recommended way is to
/// use `VoipId::new()` to generate a random one. If that function is not available for you,
/// you need to activate this crate's `rand` Cargo feature.
#[ruma_id]
pub struct VoipId;

impl VoipId {
    /// Creates a random VoIP identifier.
    ///
    /// This will currently be a UUID without hyphens, but no guarantees are made about the
    /// structure of client secrets generated from this function.
    #[cfg(feature = "rand")]
    pub fn new() -> Self {
        let id = uuid::Uuid::new_v4();
        Self::from_string_unchecked(id.simple().to_string())
    }
}
