//! Types for the *m.room.redaction* event.

use ruma_events_macros::ruma_event;
use ruma_identifiers::EventId;

ruma_event! {
    /// A redaction of an event.
    RedactionEvent {
        kind: RoomEvent,
        event_type: RoomRedaction,
        fields: {
            /// The ID of the event that was redacted.
            pub redacts: EventId,
        },
        content: {
            /// The reason for the redaction, if any.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub reason: Option<String>,
        }
    }
}
