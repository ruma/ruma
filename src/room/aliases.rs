//! Types for the *m.room.aliases* event.

use StateEvent;

/// Informs the room about what room aliases it has been given.
pub type AliasesEvent = StateEvent<AliasesEventContent>;

/// The payload of an `AliasesEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct AliasesEventContent {
    /// A list of room aliases.
    pub aliases: Vec<String>,
}
