//! Types for custom events outside of the Matrix specification.

use serde::Serialize;
use serde_json::{value::RawValue as RawJsonValue, Value as JsonValue};

use crate::{
    BasicEventContent, EphemeralRoomEventContent, EventContent, MessageEventContent,
    RoomEventContent, StateEventContent,
};

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
