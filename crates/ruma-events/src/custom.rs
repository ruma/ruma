//! Types for custom events outside of the Matrix specification.

use std::collections::BTreeMap;

use ruma_identifiers::RoomVersionId;
use serde::Serialize;
use serde_json::{value::RawValue as RawJsonValue, Value as JsonValue};

use crate::{
    EphemeralRoomEventContent, EventContent, GlobalAccountDataEventContent, HasDeserializeFields,
    MessageEventContent, RedactContent, RedactedEventContent, RedactedMessageEventContent,
    RedactedStateEventContent, RoomAccountDataEventContent, StateEventContent,
    ToDeviceEventContent,
};

/// A custom event's type and `content` JSON object.
#[derive(Clone, Debug, Serialize)]
#[allow(clippy::exhaustive_structs)]
pub struct CustomEventContent {
    /// The event type string.
    #[serde(skip)]
    pub event_type: String,

    /// The actual `content` JSON object.
    #[serde(flatten)]
    pub data: BTreeMap<String, JsonValue>,
}

impl RedactContent for CustomEventContent {
    type Redacted = RedactedCustomEventContent;

    fn redact(self, _: &RoomVersionId) -> RedactedCustomEventContent {
        RedactedCustomEventContent { event_type: self.event_type }
    }
}

impl EventContent for CustomEventContent {
    fn event_type(&self) -> &str {
        &self.event_type
    }

    fn from_parts(event_type: &str, content: &RawJsonValue) -> serde_json::Result<Self> {
        let data = serde_json::from_str(content.get())?;
        Ok(Self { event_type: event_type.to_owned(), data })
    }
}

// A custom event must satisfy all of the event content marker traits since
// they can be used for any event kind.
impl GlobalAccountDataEventContent for CustomEventContent {}

impl RoomAccountDataEventContent for CustomEventContent {}

impl ToDeviceEventContent for CustomEventContent {}

impl EphemeralRoomEventContent for CustomEventContent {}

impl MessageEventContent for CustomEventContent {}

impl StateEventContent for CustomEventContent {}

/// A custom event that has been redacted.
#[derive(Clone, Debug, Serialize)]
#[allow(clippy::exhaustive_structs)]
pub struct RedactedCustomEventContent {
    // This field is marked skipped but will be present because deserialization
    // passes the `type` field of the JSON event to the events `EventContent::from_parts` method.
    /// The event type string for this custom event "m.whatever".
    #[serde(skip)]
    pub event_type: String,
}

impl EventContent for RedactedCustomEventContent {
    fn event_type(&self) -> &str {
        &self.event_type
    }

    fn from_parts(event_type: &str, _content: &RawJsonValue) -> serde_json::Result<Self> {
        Ok(Self { event_type: event_type.to_owned() })
    }
}

impl RedactedEventContent for RedactedCustomEventContent {
    fn empty(event_type: &str) -> serde_json::Result<Self> {
        Ok(Self { event_type: event_type.to_owned() })
    }

    fn has_serialize_fields(&self) -> bool {
        false
    }

    fn has_deserialize_fields() -> HasDeserializeFields {
        HasDeserializeFields::False
    }
}

impl RedactedMessageEventContent for RedactedCustomEventContent {}

impl RedactedStateEventContent for RedactedCustomEventContent {}

/// A custom event's type. Used for event enum `_Custom` variants.
#[doc(hidden)]
// FIXME: Serialize shouldn't be required here, but it's currently a supertrait of EventContent
#[derive(Clone, Debug, Serialize)]
#[allow(clippy::exhaustive_structs)]
pub struct _CustomEventContent {
    #[serde(skip)]
    event_type: Box<str>,
}

impl RedactContent for _CustomEventContent {
    type Redacted = Self;

    fn redact(self, _: &RoomVersionId) -> Self {
        self
    }
}

impl EventContent for _CustomEventContent {
    fn event_type(&self) -> &str {
        &self.event_type
    }

    fn from_parts(event_type: &str, _content: &RawJsonValue) -> serde_json::Result<Self> {
        Ok(Self { event_type: event_type.into() })
    }
}

impl RedactedEventContent for _CustomEventContent {
    fn empty(event_type: &str) -> serde_json::Result<Self> {
        Ok(Self { event_type: event_type.into() })
    }

    fn has_serialize_fields(&self) -> bool {
        false
    }

    fn has_deserialize_fields() -> HasDeserializeFields {
        HasDeserializeFields::False
    }
}

impl GlobalAccountDataEventContent for _CustomEventContent {}
impl RoomAccountDataEventContent for _CustomEventContent {}
impl ToDeviceEventContent for _CustomEventContent {}
impl EphemeralRoomEventContent for _CustomEventContent {}
impl MessageEventContent for _CustomEventContent {}
impl StateEventContent for _CustomEventContent {}
impl RedactedMessageEventContent for _CustomEventContent {}
impl RedactedStateEventContent for _CustomEventContent {}
