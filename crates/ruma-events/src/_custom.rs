use ruma_common::{
    OwnedTransactionId, TransactionId, room_version_rules::RedactionRules, serde::CanBeEmpty,
};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;

use super::{
    EphemeralRoomEventContent, EphemeralRoomEventType, EventContentFromType,
    GlobalAccountDataEventContent, GlobalAccountDataEventType, MessageLikeEventContent,
    MessageLikeEventType, RedactContent, RedactedMessageLikeEventContent,
    RoomAccountDataEventContent, RoomAccountDataEventType, StateEventContent, StateEventType,
    StaticStateEventContent, ToDeviceEventContent, ToDeviceEventType,
};
use crate::EventUnsignedData;

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

        impl EventContentFromType for $i {
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

            fn redact(self, _: &RedactionRules) -> Self {
                self
            }
        }
    };
}

custom_event_content!(CustomGlobalAccountDataEventContent, GlobalAccountDataEventType);
impl GlobalAccountDataEventContent for CustomGlobalAccountDataEventContent {
    fn event_type(&self) -> GlobalAccountDataEventType {
        self.event_type[..].into()
    }
}

custom_event_content!(CustomRoomAccountDataEventContent, RoomAccountDataEventType);
impl RoomAccountDataEventContent for CustomRoomAccountDataEventContent {
    fn event_type(&self) -> RoomAccountDataEventType {
        self.event_type[..].into()
    }
}

custom_event_content!(CustomEphemeralRoomEventContent, EphemeralRoomEventType);
impl EphemeralRoomEventContent for CustomEphemeralRoomEventContent {
    fn event_type(&self) -> EphemeralRoomEventType {
        self.event_type[..].into()
    }
}

custom_room_event_content!(CustomMessageLikeEventContent, MessageLikeEventType);
impl MessageLikeEventContent for CustomMessageLikeEventContent {
    fn event_type(&self) -> MessageLikeEventType {
        self.event_type[..].into()
    }
}
impl RedactedMessageLikeEventContent for CustomMessageLikeEventContent {
    fn event_type(&self) -> MessageLikeEventType {
        self.event_type[..].into()
    }
}

custom_room_event_content!(CustomStateEventContent, StateEventType);
impl StateEventContent for CustomStateEventContent {
    type StateKey = String;

    fn event_type(&self) -> StateEventType {
        self.event_type[..].into()
    }
}
impl StaticStateEventContent for CustomStateEventContent {
    // Like `StateUnsigned`, but without `prev_content`.
    // We don't care about `prev_content` since we'd only store the event type that is the same
    // as in the content.
    type Unsigned = CustomStateUnsigned;
}

/// Like `StateUnsigned`, but with the minimal set of fields that we actually need.
#[derive(Clone, Debug, Default, Deserialize)]
#[non_exhaustive]
pub struct CustomStateUnsigned {
    /// The client-supplied transaction ID, if the client being given the event is the same one
    /// which sent it.
    transaction_id: Option<OwnedTransactionId>,

    /// The event that redacted this event, if any.
    redacted_because: Option<serde::de::IgnoredAny>,
}

impl CanBeEmpty for CustomStateUnsigned {
    fn is_empty(&self) -> bool {
        true
    }
}

impl EventUnsignedData for CustomStateUnsigned {
    fn transaction_id(&self) -> Option<&TransactionId> {
        self.transaction_id.as_deref()
    }

    fn is_redacted(&self) -> bool {
        self.redacted_because.is_some()
    }
}

custom_event_content!(CustomToDeviceEventContent, ToDeviceEventType);
impl ToDeviceEventContent for CustomToDeviceEventContent {
    fn event_type(&self) -> ToDeviceEventType {
        self.event_type[..].into()
    }
}
