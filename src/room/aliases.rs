//! Types for the *m.room.aliases* event.

use core::EventType;

/// Informs the room about what room aliases it has been given.
pub struct AliasesEvent {
    content: AliasesEventContent,
    event_id: String,
    event_type: EventType,
    prev_content: Option<AliasesEventContent>,
    room_id: String,
    /// The homeserver domain which owns these room aliases.
    state_key: String,
    user_id: String,
}

/// The payload of an `AliasesEvent`.
pub struct AliasesEventContent {
    /// A list of room aliases.
    aliases: Vec<String>,
}
