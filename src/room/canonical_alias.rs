//! Types for the *m.room.canonical_alias* event.

use ruma_identifiers::RoomAliasId;
use serde_derive::{Deserialize, Serialize};

state_event! {
    /// Informs the room as to which alias is the canonical one.
    pub struct CanonicalAliasEvent(CanonicalAliasEventContent) {}
}

/// The payload of a `CanonicalAliasEvent`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CanonicalAliasEventContent {
    /// The canonical alias.
    pub alias: RoomAliasId,
}
