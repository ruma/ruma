//! Types for the *m.room.redaction* event.

use ruma_identifiers::EventId;

room_event! {
    /// A redaction of an event.
    pub struct RedactionEvent(RedactionEventContent) {
        /// The ID of the event that was redacted.
        pub redacts: EventId
    }
}

/// The payload of a `RedactionEvent`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RedactionEventContent {
    /// The reason for the redaction, if any.
    #[serde(skip_serializing_if="Option::is_none")]
    pub reason: Option<String>,
}
