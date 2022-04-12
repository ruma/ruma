use serde::Serialize;
use serde_json::value::RawValue as RawJsonValue;

use super::{
    EphemeralRoomEventType, EventContent, GlobalAccountDataEventType, HasDeserializeFields,
    MessageLikeEventType, RedactContent, RedactedEventContent, RoomAccountDataEventType,
    StateEventContent, StateEventType, ToDeviceEventType,
};
use crate::RoomVersionId;

macro_rules! custom_event_content {
    ($i:ident, $evt:ident) => {
        /// A custom event's type. Used for event enum `_Custom` variants.
        // FIXME: Serialize shouldn't be required here, but it's currently a supertrait of
        // EventContent
        #[derive(Clone, Debug, Serialize)]
        #[allow(clippy::exhaustive_structs)]
        pub struct $i {
            #[serde(skip)]
            event_type: Box<str>,
        }

        impl EventContent for $i {
            type EventType = $evt;

            fn event_type(&self) -> Self::EventType {
                self.event_type[..].into()
            }

            fn from_parts(event_type: &str, _content: &RawJsonValue) -> serde_json::Result<Self> {
                Ok(Self { event_type: event_type.into() })
            }
        }
    };
}

macro_rules! custom_room_event_content {
    ($i:ident, $evt:ident) => {
        custom_event_content!($i, $evt);

        impl RedactContent for $i {
            type Redacted = Self;

            fn redact(self, _: &RoomVersionId) -> Self {
                self
            }
        }

        impl RedactedEventContent for $i {
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
    };
}

custom_event_content!(CustomGlobalAccountDataEventContent, GlobalAccountDataEventType);
custom_event_content!(CustomRoomAccountDataEventContent, RoomAccountDataEventType);
custom_event_content!(CustomEphemeralRoomEventContent, EphemeralRoomEventType);
custom_room_event_content!(CustomMessageLikeEventContent, MessageLikeEventType);
custom_room_event_content!(CustomStateEventContent, StateEventType);
custom_event_content!(CustomToDeviceEventContent, ToDeviceEventType);

impl StateEventContent for CustomStateEventContent {
    type StateKey = String;
}
