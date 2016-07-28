//! Types for the *m.room.redaction* event.

use ruma_identifiers::EventId;

use RoomEvent;

/// A redaction of an event.
pub type RedactionEvent = RoomEvent<RedactionEventContent, RedactionEventExtraContent>;

/// The payload of a `RedactionEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct RedactionEventContent {
    /// The reason for the redaction, if any.
    pub reason: Option<String>,
}

/// Extra content for a `RedactionEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct RedactionEventExtraContent {
    /// The ID of the event that was redacted.
    pub redacts: EventId,
}
