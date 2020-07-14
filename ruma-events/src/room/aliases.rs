//! Types for the *m.room.aliases* event.

use ruma_events_macros::StateEventContent;
use ruma_identifiers::RoomAliasId;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;

use crate::{EventContent, RedactedEventContent, RedactedStateEventContent, StateEvent};

/// Informs the room about what room aliases it has been given.
pub type AliasesEvent = StateEvent<AliasesEventContent>;

/// The payload for `AliasesEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[ruma_event(type = "m.room.aliases")]
#[ruma_event(custom_redacted)]
pub struct AliasesEventContent {
    /// A list of room aliases.
    pub aliases: Vec<RoomAliasId>,
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

    fn has_optional_content() -> bool {
        true
    }

    fn has_deserialize_fields() -> bool {
        true
    }
}

impl RedactedStateEventContent for RedactedAliasesEventContent {}
