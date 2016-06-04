//! Types for the *m.room.canonical_alias* event.

use core::EventType;

/// Informs the room as to which alias is the canonical one.
pub struct CanonicalAliasEvent {
    content: CanonicalAliasEventContent,
    event_id: String,
    event_type: EventType,
    prev_content: Option<CanonicalAliasEventContent>,
    room_id: String,
    state_key: String,
    user_id: String,
}

/// The payload of a `CanonicalAliasEvent`.
pub struct CanonicalAliasEventContent {
    /// The canonical alias.
    alias: String,
}
