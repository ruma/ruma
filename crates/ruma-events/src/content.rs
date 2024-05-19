use std::fmt;

use ruma_common::serde::{CanBeEmpty, Raw};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_str as from_json_str, value::RawValue as RawJsonValue};

use super::{
    EphemeralRoomEventType, GlobalAccountDataEventType, MessageLikeEventType,
    RoomAccountDataEventType, StateEventType, ToDeviceEventType,
};
use crate::{
    EphemeralRoomEvent, MessageLikeEvent, RedactContent, RedactedMessageLikeEvent,
    RedactedStateEvent, RedactedSyncMessageLikeEvent, RedactedSyncStateEvent, StateEvent,
    SyncEphemeralRoomEvent, SyncMessageLikeEvent, SyncStateEvent,
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

/// Extension trait for [`Raw<T>`].
pub trait RawEventContentExt<T: EventContentFromType> {
    /// Try to deserialize the JSON as an event's content with the given event type.
    fn deserialize_with_type(&self, event_type: T::EventType) -> serde_json::Result<T>;
}

pub trait RawRoomEventExt {
    type SyncEvent;

    fn to_sync_event(&self) -> Raw<Self::SyncEvent>;
}

pub trait RawSyncRoomEventExt {
    type FullEvent;

    fn to_full_event(&self) -> Raw<Self::FullEvent>;
}

impl<T> RawEventContentExt<T> for Raw<T>
where
    T: EventContentFromType,
    T::EventType: fmt::Display,
{
    fn deserialize_with_type(&self, event_type: T::EventType) -> serde_json::Result<T> {
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
    type Unsigned: Clone + fmt::Debug + Default + CanBeEmpty + DeserializeOwned;
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

impl<C> RawRoomEventExt for Raw<EphemeralRoomEvent<C>>
where
    C: EphemeralRoomEventContent,
{
    type SyncEvent;

    fn to_sync_event(&self) -> Raw<Self::SyncEvent> {
        todo!()
    }
}

impl<C> RawSyncRoomEventExt for Raw<SyncEphemeralRoomEvent<C>>
where
    C: EphemeralRoomEventContent,
{
    type FullEvent;

    fn to_full_event(&self) -> Raw<Self::FullEvent> {
        todo!()
    }
}

impl<C> RawRoomEventExt for Raw<MessageLikeEvent<C>>
where
    C: MessageLikeEventContent + RedactContent,
    <C as RedactContent>::Redacted: RedactedMessageLikeEventContent,
{
    type SyncEvent;

    fn to_sync_event(&self) -> Raw<Self::SyncEvent> {
        todo!()
    }
}

impl<C> RawSyncRoomEventExt for Raw<SyncMessageLikeEvent<C>>
where
    C: MessageLikeEventContent + RedactContent,
    <C as RedactContent>::Redacted: RedactedMessageLikeEventContent,
{
    type FullEvent;

    fn to_full_event(&self) -> Raw<Self::FullEvent> {
        todo!()
    }
}

impl<C> RawRoomEventExt for Raw<RedactedMessageLikeEvent<C>>
where
    C: RedactedMessageLikeEventContent,
{
    type SyncEvent;

    fn to_sync_event(&self) -> Raw<Self::SyncEvent> {
        todo!()
    }
}

impl<C> RawSyncRoomEventExt for Raw<RedactedSyncMessageLikeEvent<C>>
where
    C: RedactedMessageLikeEventContent,
{
    type FullEvent;

    fn to_full_event(&self) -> Raw<Self::FullEvent> {
        todo!()
    }
}

impl<C> RawRoomEventExt for Raw<StateEvent<C>>
where
    C: StateEventContent + StaticStateEventContent + RedactContent,
    <C as RedactContent>::Redacted: RedactedStateEventContent,
{
    type SyncEvent;

    fn to_sync_event(&self) -> Raw<Self::SyncEvent> {
        todo!()
    }
}

impl<C> RawSyncRoomEventExt for Raw<SyncStateEvent<C>>
where
    C: StateEventContent + StaticStateEventContent + RedactContent,
    <C as RedactContent>::Redacted: RedactedStateEventContent,
{
    type FullEvent;

    fn to_full_event(&self) -> Raw<Self::FullEvent> {
        todo!()
    }
}

impl<C> RawRoomEventExt for Raw<RedactedStateEvent<C>>
where
    C: RedactedStateEventContent,
{
    type SyncEvent;

    fn to_sync_event(&self) -> Raw<Self::SyncEvent> {
        todo!()
    }
}

impl<C> RawSyncRoomEventExt for Raw<RedactedSyncStateEvent<C>>
where
    C: RedactedStateEventContent,
{
    type FullEvent;

    fn to_full_event(&self) -> Raw<Self::FullEvent> {
        todo!()
    }
}
