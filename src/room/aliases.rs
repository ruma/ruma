//! Types for the *m.room.aliases* event.

use core::EventType;

/// Informs the room about what room aliases it has been given.
pub struct AliasesEvent {
    pub content: AliasesEventContent,
    pub event_id: String,
    pub event_type: EventType,
    pub prev_content: Option<AliasesEventContent>,
    pub room_id: String,
    /// The homeserver domain which owns these room aliases.
    pub state_key: String,
    pub user_id: String,
}

/// The payload of an `AliasesEvent`.
pub struct AliasesEventContent {
    /// A list of room aliases.
    pub aliases: Vec<String>,
}
