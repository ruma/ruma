use std::fmt;

use ruma_common::serde::{CanBeEmpty, Raw};
use serde::{Serialize, de::DeserializeOwned};
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

/// Extension trait for [`Raw<T>`].
pub trait RawEventContentExt<T: EventContentFromType> {
    /// Try to deserialize the JSON as an event's content with the given event type.
    fn deserialize_with_type(&self, event_type: &str) -> serde_json::Result<T>;
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
{
    fn deserialize_with_type(&self, event_type: &str) -> serde_json::Result<T> {
        T::from_parts(event_type, self.json())
    }
}

/// An event content type with a statically-known event `type` value.
///
/// Note that the `TYPE` might not be the full event type. If `IsPrefix` is set to `True`, it only
/// contains the statically-known prefix of the event type.
///
/// To only support full event types, the bound `StaticEventContent<IsPrefix = False>` can be used.
pub trait StaticEventContent: Sized {
    /// The statically-known part of the event type.
    ///
    /// If this is only the prefix of the event type, it should end with `.`, which is usually used
    /// a separator in Matrix event types.
    const TYPE: &'static str;
    /// Whether the statically-known part of the event type is the prefix.
    ///
    /// Should be set to the [`True`] or [`False`] types.
    ///
    /// Ideally this should be a boolean associated constant, but [associated constant equality is
    /// unstable], so this field could not be used as a bound. Instead we use an associated type so
    /// we can rely on associated type equality.
    ///
    /// If this is set to [`False`], the `TYPE` is the full event type.
    ///
    /// [associated constant equality is unstable]: https://github.com/rust-lang/rust/issues/92827
    type IsPrefix: BooleanType;
}

/// A trait for types representing a boolean value.
pub trait BooleanType {
    /// The boolean representation of this type.
    fn as_bool() -> bool;
}

/// The equivalent of the `true` boolean.
#[non_exhaustive]
pub struct True;

impl BooleanType for True {
    fn as_bool() -> bool {
        true
    }
}

/// The equivalent of the `false` boolean.
#[non_exhaustive]
pub struct False;

impl BooleanType for False {
    fn as_bool() -> bool {
        false
    }
}

/// Content of a global account-data event.
pub trait GlobalAccountDataEventContent: Sized + Serialize {
    /// Get the event's type, like `m.push_rules`.
    fn event_type(&self) -> GlobalAccountDataEventType;
}

/// Content of a room-specific account-data event.
pub trait RoomAccountDataEventContent: Sized + Serialize {
    /// Get the event's type, like `m.tag`.
    fn event_type(&self) -> RoomAccountDataEventType;
}

/// Content of an ephemeral room event.
pub trait EphemeralRoomEventContent: Sized + Serialize {
    /// Get the event's type, like `m.receipt`.
    fn event_type(&self) -> EphemeralRoomEventType;
}

/// Content of a non-redacted message-like event.
pub trait MessageLikeEventContent: Sized + Serialize {
    /// Get the event's type, like `m.room.message`.
    fn event_type(&self) -> MessageLikeEventType;
}

/// Content of a redacted message-like event.
pub trait RedactedMessageLikeEventContent: Sized + Serialize {
    /// Get the event's type, like `m.room.message`.
    fn event_type(&self) -> MessageLikeEventType;
}

/// Content of a non-redacted state event.
pub trait StateEventContent: Sized + Serialize {
    /// The type of the event's `state_key` field.
    type StateKey: AsRef<str> + Clone + fmt::Debug + DeserializeOwned + Serialize;

    /// Get the event's type, like `m.room.name`.
    fn event_type(&self) -> StateEventType;
}

/// Content of a non-redacted state event with a corresponding possibly-redacted type.
pub trait StaticStateEventContent: StateEventContent {
    /// The possibly redacted form of the event's content.
    type PossiblyRedacted: PossiblyRedactedStateEventContent;

    /// The type of the event's `unsigned` field.
    type Unsigned: Clone + fmt::Debug + Default + CanBeEmpty + DeserializeOwned;
}

/// Content of a redacted state event.
pub trait RedactedStateEventContent: Sized + Serialize {
    /// The type of the event's `state_key` field.
    type StateKey: AsRef<str> + Clone + fmt::Debug + DeserializeOwned + Serialize;

    /// Get the event's type, like `m.room.name`.
    fn event_type(&self) -> StateEventType;
}

/// Content of a state event.
pub trait PossiblyRedactedStateEventContent: Sized + Serialize {
    /// The type of the event's `state_key` field.
    type StateKey: AsRef<str> + Clone + fmt::Debug + DeserializeOwned + Serialize;

    /// Get the event's type, like `m.room.name`.
    fn event_type(&self) -> StateEventType;
}

/// Content of a to-device event.
pub trait ToDeviceEventContent: Sized + Serialize {
    /// Get the event's type, like `m.room_key`.
    fn event_type(&self) -> ToDeviceEventType;
}

/// Event content that can be deserialized with its event type.
pub trait EventContentFromType: Sized {
    /// Constructs this event content from the given event type and JSON.
    fn from_parts(event_type: &str, content: &RawJsonValue) -> serde_json::Result<Self>;
}

impl<T> EventContentFromType for T
where
    T: StaticEventContent + DeserializeOwned,
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
