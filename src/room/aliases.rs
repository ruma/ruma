//! Types for the *m.room.aliases* event.

use ruma_identifiers::RoomAliasId;

state_event! {
    /// Informs the room about what room aliases it has been given.
    pub struct AliasesEvent(AliasesEventContent) {}
}

/// The payload of an `AliasesEvent`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AliasesEventContent {
    /// A list of room aliases.
    pub aliases: Vec<RoomAliasId>,
}
