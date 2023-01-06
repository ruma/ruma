use std::fmt;

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_str as from_json_str, value::RawValue as RawJsonValue};

use crate::serde::{CanBeEmpty, Raw};

use super::{
    EphemeralRoomEventType, GlobalAccountDataEventType, MessageLikeEventType, RedactContent,
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

    /// Constructs the given event content.
    #[doc(hidden)]
    fn from_parts(event_type: &str, content: &RawJsonValue) -> serde_json::Result<Self>;
}

impl<T> Raw<T>
where
    T: EventContent,
    T::EventType: fmt::Display,
{
    /// Try to deserialize the JSON as an event's content.
    pub fn deserialize_content(&self, event_type: T::EventType) -> serde_json::Result<T> {
        T::from_parts(&event_type.to_string(), self.json())
    }
}

/// The base trait that all redacted event content types implement.
///
/// This trait's associated functions and methods should not be used to build
/// redacted events, prefer the `redact` method on `AnyStateEvent` and
/// `AnyMessageLikeEvent` and their "sync" and "stripped" counterparts.
/// The `RedactedEventContent` trait is an implementation detail, ruma makes no
/// API guarantees.
pub trait RedactedEventContent: EventContent {}

/// Trait for abstracting over event content structs.
///
/// … but *not* enums which don't always have an event type and kind (e.g. message vs state) that's
/// fixed / known at compile time.
pub trait StaticEventContent: EventContent {
    /// The event's "kind".
    ///
    /// See the type's documentation.
    const KIND: EventKind;

    /// The event type.
    const TYPE: &'static str;
}

/// The "kind" of an event.
///
/// This corresponds directly to the event content marker traits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum EventKind {
    /// Global account data event kind.
    GlobalAccountData,

    /// Room account data event kind.
    RoomAccountData,

    /// Ephemeral room event kind.
    EphemeralRoomData,

    /// Message-like event kind.
    ///
    /// Since redacted / non-redacted message-like events are used in the same places but have
    /// different sets of fields, these two variations are treated as two closely-related event
    /// kinds.
    MessageLike {
        /// Redacted variation?
        redacted: bool,
    },

    /// State event kind.
    ///
    /// Since redacted / non-redacted state events are used in the same places but have different
    /// sets of fields, these two variations are treated as two closely-related event kinds.
    State {
        /// Redacted variation?
        redacted: bool,
    },

    /// To-device event kind.
    ToDevice,

    /// Presence event kind.
    Presence,
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

/// Content of a message-like event.
pub trait MessageLikeEventContent: EventContent<EventType = MessageLikeEventType> {}

/// Content of a redacted message-like event.
pub trait RedactedMessageLikeEventContent: MessageLikeEventContent + RedactedEventContent {}

/// Content of a state event.
pub trait StateEventContent: EventContent<EventType = StateEventType> {
    /// The type of the event's `state_key` field.
    type StateKey: AsRef<str> + Clone + fmt::Debug + DeserializeOwned + Serialize;
}

/// Content of a non-redacted state event.
pub trait OriginalStateEventContent: StateEventContent + RedactContent {
    /// The type of the event's `unsigned` field.
    type Unsigned: Clone + fmt::Debug + Default + CanBeEmpty + DeserializeOwned;

    /// The possibly redacted form of the event's content.
    type PossiblyRedacted: StateEventContent;
}

/// Content of a redacted state event.
pub trait RedactedStateEventContent: StateEventContent + RedactedEventContent {}

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
