use serde::Serialize;
use serde_json::value::RawValue as RawJsonValue;

use super::{
    EphemeralRoomEventContent, EphemeralRoomEventType, EventContent, GlobalAccountDataEventContent,
    GlobalAccountDataEventType, MessageLikeEventContent, MessageLikeEventType,
    OriginalStateEventContent, RedactContent, RedactedEventContent,
    RedactedMessageLikeEventContent, RedactedStateEventContent, RoomAccountDataEventContent,
    RoomAccountDataEventType, StateEventContent, StateEventType, StateUnsigned,
    ToDeviceEventContent, ToDeviceEventType,
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

        impl RedactedEventContent for $i {}
    };
}

custom_event_content!(CustomGlobalAccountDataEventContent, GlobalAccountDataEventType);
impl GlobalAccountDataEventContent for CustomGlobalAccountDataEventContent {}

custom_event_content!(CustomRoomAccountDataEventContent, RoomAccountDataEventType);
impl RoomAccountDataEventContent for CustomRoomAccountDataEventContent {}

custom_event_content!(CustomEphemeralRoomEventContent, EphemeralRoomEventType);
impl EphemeralRoomEventContent for CustomEphemeralRoomEventContent {}

custom_room_event_content!(CustomMessageLikeEventContent, MessageLikeEventType);
impl MessageLikeEventContent for CustomMessageLikeEventContent {}
impl RedactedMessageLikeEventContent for CustomMessageLikeEventContent {}

custom_room_event_content!(CustomStateEventContent, StateEventType);
impl StateEventContent for CustomStateEventContent {
    type StateKey = String;
}
impl OriginalStateEventContent for CustomStateEventContent {
    type Unsigned = StateUnsigned<Self>;
    type PossiblyRedacted = Self;
}
impl RedactedStateEventContent for CustomStateEventContent {}

custom_event_content!(CustomToDeviceEventContent, ToDeviceEventType);
impl ToDeviceEventContent for CustomToDeviceEventContent {}
