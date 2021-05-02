//! Types for the *m.room.aliases* event.

use ruma_events_macros::StateEventContent;
use ruma_identifiers::{RoomAliasId, RoomVersionId};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;

use crate::{
    EventContent, HasDeserializeFields, RedactedEventContent, RedactedStateEventContent, StateEvent,
};

/// Informs the room about what room aliases it has been given.
pub type AliasesEvent = StateEvent<AliasesEventContent>;

/// The payload for `AliasesEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.aliases", custom_redacted)]
pub struct AliasesEventContent {
    /// A list of room aliases.
    pub aliases: Vec<RoomAliasId>,
}

impl AliasesEventContent {
    /// Create an `AliasesEventContent` from the given aliases.
    pub fn new(aliases: Vec<RoomAliasId>) -> Self {
        Self { aliases }
    }

    /// Redact an `AliasesEventContent` according to current Matrix spec.
    pub fn redact(self, version: RoomVersionId) -> RedactedAliasesEventContent {
        // We compare the long way to avoid pre version 6 behavior if/when
        // a new room version is introduced.
        let aliases = match version {
            RoomVersionId::Version1
            | RoomVersionId::Version2
            | RoomVersionId::Version3
            | RoomVersionId::Version4
            | RoomVersionId::Version5 => Some(self.aliases),
            _ => None,
        };

        RedactedAliasesEventContent { aliases }
    }
}

/// An aliases event that has been redacted.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RedactedAliasesEventContent {
    /// A list of room aliases.
    ///
    /// According to the Matrix spec version 1 redaction rules allowed this field to be
    /// kept after redaction, this was changed in version 6.
    pub aliases: Option<Vec<RoomAliasId>>,
}

impl EventContent for RedactedAliasesEventContent {
    fn event_type(&self) -> &str {
        "m.room.aliases"
    }

    fn from_parts(event_type: &str, content: Box<RawJsonValue>) -> Result<Self, serde_json::Error> {
        if event_type != "m.room.aliases" {
            return Err(::serde::de::Error::custom(format!(
                "expected event type `m.room.aliases`, found `{}`",
                event_type
            )));
        }

        serde_json::from_str(content.get())
    }
}

// Since this redacted event has fields we leave the default `empty` method
// that will error if called.
impl RedactedEventContent for RedactedAliasesEventContent {
    fn has_serialize_fields(&self) -> bool {
        self.aliases.is_some()
    }

    fn has_deserialize_fields() -> HasDeserializeFields {
        HasDeserializeFields::Optional
    }
}

impl RedactedStateEventContent for RedactedAliasesEventContent {}
