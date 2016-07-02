//! Types for the *m.room.canonical_alias* event.

use StateEvent;

/// Informs the room as to which alias is the canonical one.
pub type CanonicalAliasEvent = StateEvent<CanonicalAliasEventContent, ()>;

/// The payload of a `CanonicalAliasEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct CanonicalAliasEventContent {
    /// The canonical alias.
    pub alias: String,
}
