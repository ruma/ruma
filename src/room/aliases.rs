//! Types for the *m.room.aliases* event.

use ruma_events_macros::StateEventContent;
use ruma_identifiers::RoomAliasId;
use serde::{Deserialize, Serialize};

/// Informs the room about what room aliases it has been given.
#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[ruma_event(type = "m.room.aliases")]
pub struct AliasesEventContent {
    /// A list of room aliases.
    pub aliases: Vec<RoomAliasId>,
}
