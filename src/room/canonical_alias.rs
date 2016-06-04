//! Types for the *m.room.canonical_alias* event.

use core::EventType;

/// Informs the room as to which alias is the canonical one.
pub struct CanonicalAliasEvent {
    pub content: CanonicalAliasEventContent,
    pub event_id: String,
    pub event_type: EventType,
    pub prev_content: Option<CanonicalAliasEventContent>,
    pub room_id: String,
    pub state_key: String,
    pub user_id: String,
}

/// The payload of a `CanonicalAliasEvent`.
pub struct CanonicalAliasEventContent {
    /// The canonical alias.
    pub alias: String,
}
