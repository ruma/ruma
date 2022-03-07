use ruma_identifiers::RoomVersionId;
use serde::Serialize;
use serde_json::value::RawValue as RawJsonValue;

use super::{
    EphemeralRoomEventContent, EventContent, HasDeserializeFields, MessageLikeEventContent,
    RedactContent, RedactedEventContent, RedactedMessageLikeEventContent,
    RedactedStateEventContent, StateEventContent, ToDeviceEventContent,
};

/// A custom event's type. Used for event enum `_Custom` variants.
// FIXME: Serialize shouldn't be required here, but it's currently a supertrait of EventContent
#[derive(Clone, Debug, Serialize)]
#[allow(clippy::exhaustive_structs)]
pub struct CustomEventContent {
    #[serde(skip)]
    event_type: Box<str>,
}

impl RedactContent for CustomEventContent {
    type Redacted = Self;

    fn redact(self, _: &RoomVersionId) -> Self {
        self
    }
}

impl EventContent for CustomEventContent {
    fn event_type(&self) -> &str {
        &self.event_type
    }

    fn from_parts(event_type: &str, _content: &RawJsonValue) -> serde_json::Result<Self> {
        Ok(Self { event_type: event_type.into() })
    }
}

impl RedactedEventContent for CustomEventContent {
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

impl ToDeviceEventContent for CustomEventContent {}
impl EphemeralRoomEventContent for CustomEventContent {}
impl MessageLikeEventContent for CustomEventContent {}
impl StateEventContent for CustomEventContent {}
impl RedactedMessageLikeEventContent for CustomEventContent {}
impl RedactedStateEventContent for CustomEventContent {}
