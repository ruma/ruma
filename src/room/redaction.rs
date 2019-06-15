//! Types for the *m.room.redaction* event.

use ruma_identifiers::EventId;
use serde::{Deserialize, Serialize};

room_event! {
    /// A redaction of an event.
    pub struct RedactionEvent(RedactionEventContent) {
        /// The ID of the event that was redacted.
        pub redacts: EventId
    }
}

/// The payload of a `RedactionEvent`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RedactionEventContent {
    /// The reason for the redaction, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
