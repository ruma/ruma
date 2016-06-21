//! Types for the *m.room.redaction* event.

use events::EventType;

/// A redaction of an event.
#[derive(Debug, Deserialize, Serialize)]
pub struct RedactionEvent {
    pub content: RedactionEventContent,
    pub event_id: String,
    #[serde(rename="type")]
    pub event_type: EventType,
    /// The ID of the event that was redacted.
    pub redacts: String,
    pub room_id: String,
    #[serde(rename="sender")]
    pub user_id: String,
}

/// The payload of a `RedactionEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct RedactionEventContent {
    /// The reason for the redaction, if any.
    pub reason: Option<String>,
}
