//! Types for the *m.room.redaction* event.

use std::{collections::BTreeMap, time::SystemTime};

use ruma_events_macros::{Event, EventContent};
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{Deserialize, Serialize};
use serde_json::{value::RawValue as RawJsonValue, Value as JsonValue};

use crate::UnsignedData;

/// Redaction event.
#[derive(Clone, Debug, Event)]
pub struct RedactionEvent {
    /// Data specific to the event type.
    pub content: RedactionEventContent,

    /// The ID of the event that was redacted.
    pub redacts: EventId,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// The ID of the room associated with this event.
    pub room_id: RoomId,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: UnsignedData,
}

/// Redaction event without a `room_id`.
#[derive(Clone, Debug, Event)]
pub struct RedactionEventStub {
    /// Data specific to the event type.
    pub content: RedactionEventContent,

    /// The ID of the event that was redacted.
    pub redacts: EventId,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: UnsignedData,
}

/// A redaction of an event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "m.room.redaction")]
pub struct RedactionEventContent {
    /// The reason for the redaction, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl ruma_events::RoomEventContent for RedactionEventContent {}

impl ruma_events::MessageEventContent for RedactionEventContent {}

/// The content of any event that has been redacted.
///
/// This does not represent the redaction event itself but, the removal of
/// some events content via redaction.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RedactedContent {
    /// The redacted events type.
    #[serde(rename = "type", skip_serializing)]
    pub event_type: String,

    /// The reason for the redaction, if any.
    ///
    /// This field is copied from the redaction event that affected this event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    // TODO how do we want to handle this...
    /// The keys that are allowed to remain inside of the `content` field
    /// after the original event has been redacted.
    #[serde(flatten)]
    pub left_over_keys: BTreeMap<String, JsonValue>,
}

#[derive(Deserialize)]
struct RedactHelper {
    reason: Option<String>,

    // This allows the rest of the keys to be collected here.
    #[serde(flatten)]
    left_over_keys: BTreeMap<String, JsonValue>,
}

impl ruma_events::EventContent for RedactedContent {
    fn event_type(&self) -> &str {
        &self.event_type
    }

    fn from_parts(event_type: &str, content: Box<RawJsonValue>) -> Result<Self, serde_json::Error> {
        let RedactHelper { reason, left_over_keys } = serde_json::from_str(content.get())?;
        Ok(Self { event_type: event_type.to_string(), reason, left_over_keys })
    }
}

impl ruma_events::RoomEventContent for RedactedContent {}

impl ruma_events::BasicEventContent for RedactedContent {}

impl ruma_events::MessageEventContent for RedactedContent {}

impl ruma_events::StateEventContent for RedactedContent {}
