use std::fmt;

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_str as from_json_str, value::RawValue as RawJsonValue};

use crate::{
    serde::{CanBeEmpty, Raw},
    TransactionId,
};

use super::{
    EphemeralRoomEventType, GlobalAccountDataEventType, MessageLikeEventType,
    RoomAccountDataEventType, StateEventType, ToDeviceEventType,
};

/// The base trait that all event content types implement.
///
/// Use [`macros::EventContent`] to derive this traits. It is not meant to be implemented manually.
///
/// [`macros::EventContent`]: super::macros::EventContent
pub trait EventContent: Sized + Serialize {
    /// The Rust enum for the event kind's known types.
    type EventType;

    /// Get the event's type, like `m.room.message`.
    fn event_type(&self) -> Self::EventType;
}

impl<T> Raw<T>
where
    T: EventContentFromType,
    T::EventType: fmt::Display,
{
    /// Try to deserialize the JSON as an event's content with the given event type.
    pub fn deserialize_with_type(&self, event_type: T::EventType) -> serde_json::Result<T> {
        T::from_parts(&event_type.to_string(), self.json())
    }
}

/// An event content type with a statically-known event `type` value.
pub trait StaticEventContent: EventContent {
    /// The event type.
    const TYPE: &'static str;
}

/// Content of a global account-data event.
pub trait GlobalAccountDataEventContent:
    EventContent<EventType = GlobalAccountDataEventType>
{
}

/// Content of a room-specific account-data event.
pub trait RoomAccountDataEventContent: EventContent<EventType = RoomAccountDataEventType> {}

/// Content of an ephemeral room event.
pub trait EphemeralRoomEventContent: EventContent<EventType = EphemeralRoomEventType> {}

/// Content of a non-redacted message-like event.
pub trait MessageLikeEventContent: EventContent<EventType = MessageLikeEventType> {}

/// Content of a redacted message-like event.
pub trait RedactedMessageLikeEventContent: EventContent<EventType = MessageLikeEventType> {}

/// Content of a non-redacted state event.
pub trait StateEventContent: EventContent<EventType = StateEventType> {
    /// The type of the event's `state_key` field.
    type StateKey: AsRef<str> + Clone + fmt::Debug + DeserializeOwned + Serialize;
}

/// Content of a non-redacted state event with a corresponding possibly-redacted type.
pub trait StaticStateEventContent: StateEventContent {
    /// The possibly redacted form of the event's content.
    type PossiblyRedacted: PossiblyRedactedStateEventContent;

    /// The type of the event's `unsigned` field.
    type Unsigned: Clone
        + fmt::Debug
        + Default
        + CanBeEmpty
        + DeserializeOwned
        + OriginalStateUnsigned;
}

/// Content of a redacted state event.
pub trait RedactedStateEventContent: EventContent<EventType = StateEventType> {
    /// The type of the event's `state_key` field.
    type StateKey: AsRef<str> + Clone + fmt::Debug + DeserializeOwned + Serialize;
}

/// Content of a state event.
pub trait PossiblyRedactedStateEventContent: EventContent<EventType = StateEventType> {
    /// The type of the event's `state_key` field.
    type StateKey: AsRef<str> + Clone + fmt::Debug + DeserializeOwned + Serialize;
}

/// Content of a to-device event.
pub trait ToDeviceEventContent: EventContent<EventType = ToDeviceEventType> {}

/// Event content that can be deserialized with its event type.
pub trait EventContentFromType: EventContent {
    /// Constructs this event content from the given event type and JSON.
    #[doc(hidden)]
    fn from_parts(event_type: &str, content: &RawJsonValue) -> serde_json::Result<Self>;
}

impl<T> EventContentFromType for T
where
    T: EventContent + DeserializeOwned,
{
    fn from_parts(_event_type: &str, content: &RawJsonValue) -> serde_json::Result<Self> {
        from_json_str(content.get())
    }
}

/// Unsigned data of an unredacted event.
pub trait OriginalStateUnsigned {
    /// The transaction ID if this event was sent from this session.
    fn transaction_id(&self) -> Option<&TransactionId>;
}
