//! Types for the `m.room.aliases` event.

use ruma_common::{OwnedRoomAliasId, OwnedServerName, RoomVersionId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{EventContent, RedactContent, RedactedStateEventContent, StateEventType};

/// The content of an `m.room.aliases` event.
///
/// Informs the room about what room aliases it has been given.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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

    fn redact(self, version: &RoomVersionId) -> RedactedRoomAliasesEventContent {
        // We compare the long way to avoid pre version 6 behavior if/when
        // a new room version is introduced.
        let aliases = match version {
            RoomVersionId::V1
            | RoomVersionId::V2
            | RoomVersionId::V3
            | RoomVersionId::V4
            | RoomVersionId::V5 => Some(self.aliases),
            _ => None,
        };

        RedactedRoomAliasesEventContent { aliases }
    }
}

/// An aliases event that has been redacted.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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

impl EventContent for RedactedRoomAliasesEventContent {
    type EventType = StateEventType;

    fn event_type(&self) -> StateEventType {
        StateEventType::RoomAliases
    }
}

impl RedactedStateEventContent for RedactedRoomAliasesEventContent {
    type StateKey = OwnedServerName;
}
