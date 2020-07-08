//! Types for custom events outside of the Matrix specification.

use serde::Serialize;
use serde_json::{value::RawValue as RawJsonValue, Value as JsonValue};

use crate::{
    BasicEventContent, EphemeralRoomEventContent, EventContent, MessageEventContent,
    RoomEventContent, StateEventContent,
};

/// A custom event that has been redacted.
#[derive(Clone, Debug, Serialize)]
pub struct RedactedCustomEventContent;

impl EventContent for RedactedCustomEventContent {
    fn event_type(&self) -> &str {
        // TODO how do we handle this?
        "custom"
    }

    fn from_parts(
        _event_type: &str,
        _content: Box<RawJsonValue>,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self)
    }

    fn redacted(_event_type: &str) -> Result<Self, serde_json::Error> {
        Ok(Self)
    }
}

// A redacted custom event must satisfy all of the event content marker traits since
// they can be used for any event kind.
impl RoomEventContent for RedactedCustomEventContent {}

impl BasicEventContent for RedactedCustomEventContent {}

impl EphemeralRoomEventContent for RedactedCustomEventContent {}

impl MessageEventContent for RedactedCustomEventContent {}

impl StateEventContent for RedactedCustomEventContent {}

/// A custom event's type and `content` JSON object.
#[derive(Clone, Debug, Serialize)]
pub struct CustomEventContent {
    /// The event type string.
    #[serde(skip)]
    pub event_type: String,

    /// The actual `content` JSON object.
    #[serde(flatten)]
    pub json: JsonValue,
}

impl EventContent for CustomEventContent {
    fn event_type(&self) -> &str {
        &self.event_type
    }

    fn from_parts(event_type: &str, content: Box<RawJsonValue>) -> Result<Self, serde_json::Error> {
        let json = serde_json::from_str(content.get())?;
        Ok(Self { event_type: event_type.to_string(), json })
    }
}

// A custom event must satisfy all of the event content marker traits since
// they can be used for any event kind.
impl RoomEventContent for CustomEventContent {}

impl BasicEventContent for CustomEventContent {}

impl EphemeralRoomEventContent for CustomEventContent {}

impl MessageEventContent for CustomEventContent {}

impl StateEventContent for CustomEventContent {}
