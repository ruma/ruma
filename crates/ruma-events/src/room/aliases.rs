//! Types for the `m.room.aliases` event.

use ruma_common::{OwnedRoomAliasId, OwnedServerName, room_version_rules::RedactionRules};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{RedactContent, RedactedStateEventContent, StateEventType, StaticEventContent};

/// The content of an `m.room.aliases` event.
///
/// Informs the room about what room aliases it has been given.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.room.aliases", kind = State, state_key_type = OwnedServerName, custom_redacted)]
pub struct RoomAliasesEventContent {
    /// A list of room aliases.
    pub aliases: Vec<OwnedRoomAliasId>,
}

impl RoomAliasesEventContent {
    /// Create an `RoomAliasesEventContent` from the given aliases.
    pub fn new(aliases: Vec<OwnedRoomAliasId>) -> Self {
        Self { aliases }
    }
}

impl RedactContent for RoomAliasesEventContent {
    type Redacted = RedactedRoomAliasesEventContent;

    fn redact(self, rules: &RedactionRules) -> RedactedRoomAliasesEventContent {
        let aliases = rules.keep_room_aliases_aliases.then_some(self.aliases);
        RedactedRoomAliasesEventContent { aliases }
    }
}

/// An aliases event that has been redacted.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RedactedRoomAliasesEventContent {
    /// A list of room aliases.
    ///
    /// According to the Matrix spec version 1 redaction rules allowed this field to be
    /// kept after redaction, this was changed in version 6.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<OwnedRoomAliasId>>,
}

impl RedactedRoomAliasesEventContent {
    /// Create a `RedactedAliasesEventContent` with the given aliases.
    ///
    /// This is only valid for room version 5 and below.
    pub fn new_v1(aliases: Vec<OwnedRoomAliasId>) -> Self {
        Self { aliases: Some(aliases) }
    }

    /// Create a `RedactedAliasesEventContent` with the given aliases.
    ///
    /// This is only valid for room version 6 and above.
    pub fn new_v6() -> Self {
        Self::default()
    }
}

impl RedactedStateEventContent for RedactedRoomAliasesEventContent {
    type StateKey = OwnedServerName;

    fn event_type(&self) -> StateEventType {
        StateEventType::RoomAliases
    }
}

impl StaticEventContent for RedactedRoomAliasesEventContent {
    const TYPE: &'static str = RoomAliasesEventContent::TYPE;
    type IsPrefix = <RoomAliasesEventContent as StaticEventContent>::IsPrefix;
}

impl From<RedactedRoomAliasesEventContent> for PossiblyRedactedRoomAliasesEventContent {
    fn from(value: RedactedRoomAliasesEventContent) -> Self {
        Self { aliases: value.aliases }
    }
}

impl RedactContent for PossiblyRedactedRoomAliasesEventContent {
    type Redacted = Self;

    fn redact(self, rules: &RedactionRules) -> Self {
        let aliases = self.aliases.filter(|_| rules.keep_room_aliases_aliases);
        Self { aliases }
    }
}
