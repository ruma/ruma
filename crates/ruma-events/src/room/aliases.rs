//! Types for the `m.room.aliases` event.

use ruma_common::{OwnedRoomAliasId, OwnedServerName, room_version_rules::RedactionRules};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::RedactContent;

/// The content of an `m.room.aliases` event.
///
/// Informs the room about what room aliases it has been given.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.room.aliases", kind = State, state_key_type = OwnedServerName, custom_redacted)]
pub struct RoomAliasesEventContent {
    /// A list of room aliases.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<OwnedRoomAliasId>>,
}

impl RoomAliasesEventContent {
    /// Create an `RoomAliasesEventContent` from the given aliases.
    pub fn new(aliases: Vec<OwnedRoomAliasId>) -> Self {
        Self { aliases: Some(aliases) }
    }
}

impl RedactContent for RoomAliasesEventContent {
    type Redacted = Self;

    fn redact(self, rules: &RedactionRules) -> Self {
        let aliases = self.aliases.filter(|_| rules.keep_room_aliases_aliases);
        Self { aliases }
    }
}
