use as_variant::as_variant;
use ruma_common::{
    EventId, MilliSecondsSinceUnixEpoch, RoomId, UserId,
    encryption::DeviceKeys,
    room_version_rules::RedactionRules,
    serde::{JsonCastable, JsonObject, Raw, from_raw_json_value},
};
use ruma_macros::Event;
use serde::{Deserialize, Deserializer, Serialize, ser::SerializeStruct};
use serde_json::value::RawValue as RawJsonValue;

use super::{
    AnyInitialStateEvent, EmptyStateKey, EphemeralRoomEventContent, EventContentFromType,
    GlobalAccountDataEventContent, MessageLikeEventContent, MessageLikeEventType,
    MessageLikeUnsigned, PossiblyRedactedStateEventContent, RedactContent,
    RedactedMessageLikeEventContent, RedactedStateEventContent, RedactedUnsigned,
    RedactionDeHelper, RoomAccountDataEventContent, StateEventType, StaticStateEventContent,
    ToDeviceEventContent,
};

/// A global account data event.
#[derive(Clone, Debug, Event)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct GlobalAccountDataEvent<C: GlobalAccountDataEventContent> {
    /// Data specific to the event type.
    pub content: C,
}

impl<C: GlobalAccountDataEventContent> GlobalAccountDataEvent<C> {
    /// Construct a new `GlobalAccountDataEvent` with the given content.
    pub fn new(content: C) -> Self {
        Self { content }
    }
}

impl<C: GlobalAccountDataEventContent> Serialize for GlobalAccountDataEvent<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("GlobalAccountDataEvent", 2)?;
        state.serialize_field("type", &self.content.event_type())?;
        state.serialize_field("content", &self.content)?;
        state.end()
    }
}

impl<C: GlobalAccountDataEventContent> JsonCastable<JsonObject> for GlobalAccountDataEvent<C> {}

/// A room account data event.
#[derive(Clone, Debug, Event)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RoomAccountDataEvent<C: RoomAccountDataEventContent> {
    /// Data specific to the event type.
    pub content: C,
}

impl<C: RoomAccountDataEventContent> RoomAccountDataEvent<C> {
    /// Construct a new `RoomAccountDataEvent` with the given content.
    pub fn new(content: C) -> Self {
        Self { content }
    }
}

impl<C: RoomAccountDataEventContent> Serialize for RoomAccountDataEvent<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("RoomAccountDataEvent", 2)?;
        state.serialize_field("type", &self.content.event_type())?;
        state.serialize_field("content", &self.content)?;
        state.end()
    }
}

impl<C: RoomAccountDataEventContent> JsonCastable<JsonObject> for RoomAccountDataEvent<C> {}

/// An ephemeral room event.
#[derive(Clone, Debug, Event)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct EphemeralRoomEvent<C: EphemeralRoomEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The ID of the room associated with this event.
    pub room_id: RoomId,
}

impl<C: EphemeralRoomEventContent> EphemeralRoomEvent<C> {
    /// Construct a new `EphemeralRoomEvent` with the given content and room ID.
    pub fn new(room_id: RoomId, content: C) -> Self {
        Self { content, room_id }
    }
}

impl<C: EphemeralRoomEventContent> Serialize for EphemeralRoomEvent<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("EphemeralRoomEvent", 2)?;
        state.serialize_field("type", &self.content.event_type())?;
        state.serialize_field("content", &self.content)?;
        state.serialize_field("room_id", &self.room_id)?;
        state.end()
    }
}

impl<C: EphemeralRoomEventContent> JsonCastable<SyncEphemeralRoomEvent<C>>
    for EphemeralRoomEvent<C>
{
}

impl<C: EphemeralRoomEventContent> JsonCastable<JsonObject> for EphemeralRoomEvent<C> {}

/// An ephemeral room event without a `room_id`.
#[derive(Clone, Debug, Event)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct SyncEphemeralRoomEvent<C: EphemeralRoomEventContent> {
    /// Data specific to the event type.
    pub content: C,
}

impl<C: EphemeralRoomEventContent> SyncEphemeralRoomEvent<C> {
    /// Construct a new `SyncEphemeralRoomEvent` with the given content and room ID.
    pub fn new(content: C) -> Self {
        Self { content }
    }
}

impl<C: EphemeralRoomEventContent> Serialize for SyncEphemeralRoomEvent<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("SyncEphemeralRoomEvent", 2)?;
        state.serialize_field("type", &self.content.event_type())?;
        state.serialize_field("content", &self.content)?;
        state.end()
    }
}

impl<C: EphemeralRoomEventContent> JsonCastable<JsonObject> for SyncEphemeralRoomEvent<C> {}

/// An unredacted message-like event.
///
/// `OriginalMessageLikeEvent` implements the comparison traits using only the `event_id` field, a
/// sorted list would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct OriginalMessageLikeEvent<C: MessageLikeEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique identifier for the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp on the originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The ID of the room associated with this event.
    pub room_id: RoomId,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: MessageLikeUnsigned<C>,
}

impl<C: MessageLikeEventContent> JsonCastable<OriginalSyncMessageLikeEvent<C>>
    for OriginalMessageLikeEvent<C>
{
}

impl<C: MessageLikeEventContent + RedactContent> JsonCastable<MessageLikeEvent<C>>
    for OriginalMessageLikeEvent<C>
where
    C::Redacted: RedactedMessageLikeEventContent,
{
}

impl<C: MessageLikeEventContent + RedactContent> JsonCastable<SyncMessageLikeEvent<C>>
    for OriginalMessageLikeEvent<C>
where
    C::Redacted: RedactedMessageLikeEventContent,
{
}

impl<C: MessageLikeEventContent> JsonCastable<JsonObject> for OriginalMessageLikeEvent<C> {}

/// An unredacted message-like event without a `room_id`.
///
/// `OriginalSyncMessageLikeEvent` implements the comparison traits using only the `event_id` field,
/// a sorted list would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct OriginalSyncMessageLikeEvent<C: MessageLikeEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique identifier for the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp on the originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: MessageLikeUnsigned<C>,
}

impl<C: MessageLikeEventContent + RedactContent> OriginalSyncMessageLikeEvent<C>
where
    C::Redacted: RedactedMessageLikeEventContent,
{
    pub(crate) fn into_maybe_redacted(self) -> SyncMessageLikeEvent<C> {
        SyncMessageLikeEvent::Original(self)
    }
}

impl<C: MessageLikeEventContent + RedactContent> JsonCastable<SyncMessageLikeEvent<C>>
    for OriginalSyncMessageLikeEvent<C>
where
    C::Redacted: RedactedMessageLikeEventContent,
{
}

impl<C: MessageLikeEventContent> JsonCastable<JsonObject> for OriginalSyncMessageLikeEvent<C> {}

/// A redacted message-like event.
///
/// `RedactedMessageLikeEvent` implements the comparison traits using only the `event_id` field, a
/// sorted list would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RedactedMessageLikeEvent<C: RedactedMessageLikeEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique identifier for the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp on the originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The ID of the room associated with this event.
    pub room_id: RoomId,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RedactedUnsigned,
}

impl<C: RedactedMessageLikeEventContent> JsonCastable<RedactedSyncMessageLikeEvent<C>>
    for RedactedMessageLikeEvent<C>
{
}

impl<C: MessageLikeEventContent + RedactContent> JsonCastable<MessageLikeEvent<C>>
    for RedactedMessageLikeEvent<C::Redacted>
where
    C::Redacted: RedactedMessageLikeEventContent,
{
}

impl<C: MessageLikeEventContent + RedactContent> JsonCastable<SyncMessageLikeEvent<C>>
    for RedactedMessageLikeEvent<C::Redacted>
where
    C::Redacted: RedactedMessageLikeEventContent,
{
}

impl<C: RedactedMessageLikeEventContent> JsonCastable<JsonObject> for RedactedMessageLikeEvent<C> {}

/// A redacted message-like event without a `room_id`.
///
/// `RedactedSyncMessageLikeEvent` implements the comparison traits using only the `event_id` field,
/// a sorted list would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RedactedSyncMessageLikeEvent<C: RedactedMessageLikeEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique identifier for the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp on the originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RedactedUnsigned,
}

impl<C: MessageLikeEventContent + RedactContent> JsonCastable<SyncMessageLikeEvent<C>>
    for RedactedSyncMessageLikeEvent<C::Redacted>
where
    C::Redacted: RedactedMessageLikeEventContent,
{
}

impl<C: RedactedMessageLikeEventContent> JsonCastable<JsonObject>
    for RedactedSyncMessageLikeEvent<C>
{
}

/// A possibly-redacted message-like event.
///
/// `MessageLikeEvent` implements the comparison traits using only the `event_id` field, a sorted
/// list would be sorted lexicographically based on the event's `EventId`.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Debug)]
pub enum MessageLikeEvent<C: MessageLikeEventContent + RedactContent>
where
    C::Redacted: RedactedMessageLikeEventContent,
{
    /// Original, unredacted form of the event.
    Original(OriginalMessageLikeEvent<C>),

    /// Redacted form of the event with minimal fields.
    Redacted(RedactedMessageLikeEvent<C::Redacted>),
}

impl<C: MessageLikeEventContent + RedactContent> JsonCastable<SyncMessageLikeEvent<C>>
    for MessageLikeEvent<C>
where
    C::Redacted: RedactedMessageLikeEventContent,
{
}

impl<C: MessageLikeEventContent + RedactContent> JsonCastable<JsonObject> for MessageLikeEvent<C> where
    C::Redacted: RedactedMessageLikeEventContent
{
}

/// A possibly-redacted message-like event without a `room_id`.
///
/// `SyncMessageLikeEvent` implements the comparison traits using only the `event_id` field, a
/// sorted list would be sorted lexicographically based on the event's `EventId`.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Debug)]
pub enum SyncMessageLikeEvent<C: MessageLikeEventContent + RedactContent>
where
    C::Redacted: RedactedMessageLikeEventContent,
{
    /// Original, unredacted form of the event.
    Original(OriginalSyncMessageLikeEvent<C>),

    /// Redacted form of the event with minimal fields.
    Redacted(RedactedSyncMessageLikeEvent<C::Redacted>),
}

impl<C: MessageLikeEventContent + RedactContent> JsonCastable<JsonObject>
    for SyncMessageLikeEvent<C>
where
    C::Redacted: RedactedMessageLikeEventContent,
{
}

/// An unredacted state event.
///
/// `OriginalStateEvent` implements the comparison traits using only the `event_id` field, a sorted
/// list would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct OriginalStateEvent<C: StaticStateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique identifier for the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp on the originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The ID of the room associated with this event.
    pub room_id: RoomId,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This must be a string type, and is often an empty string.
    ///
    /// A state event is keyed by its `(type, state_key)` tuple. Sending another state event with
    /// the same tuple replaces the previous one.
    pub state_key: C::StateKey,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: C::Unsigned,
}

impl<C: StaticStateEventContent> JsonCastable<OriginalSyncStateEvent<C>> for OriginalStateEvent<C> {}

impl<C: StaticStateEventContent + RedactContent> JsonCastable<StateEvent<C>>
    for OriginalStateEvent<C>
where
    C::Redacted: RedactedStateEventContent,
{
}

impl<C: StaticStateEventContent + RedactContent> JsonCastable<SyncStateEvent<C>>
    for OriginalStateEvent<C>
where
    C::Redacted: RedactedStateEventContent,
{
}

impl<C: StaticStateEventContent> JsonCastable<StrippedStateEvent<C::PossiblyRedacted>>
    for OriginalStateEvent<C>
where
    C::PossiblyRedacted: PossiblyRedactedStateEventContent,
{
}

impl<C: StaticStateEventContent> JsonCastable<JsonObject> for OriginalStateEvent<C> {}

/// An unredacted state event without a `room_id`.
///
/// `OriginalSyncStateEvent` implements the comparison traits using only the `event_id` field, a
/// sorted list would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct OriginalSyncStateEvent<C: StaticStateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique identifier for the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp on the originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This must be a string type, and is often an empty string.
    ///
    /// A state event is keyed by its `(type, state_key)` tuple. Sending another state event with
    /// the same tuple replaces the previous one.
    pub state_key: C::StateKey,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: C::Unsigned,
}

impl<C: StaticStateEventContent + RedactContent> JsonCastable<SyncStateEvent<C>>
    for OriginalSyncStateEvent<C>
where
    C::Redacted: RedactedStateEventContent,
{
}

impl<C: StaticStateEventContent> JsonCastable<StrippedStateEvent<C::PossiblyRedacted>>
    for OriginalSyncStateEvent<C>
where
    C::PossiblyRedacted: PossiblyRedactedStateEventContent,
{
}

impl<C: StaticStateEventContent> JsonCastable<JsonObject> for OriginalSyncStateEvent<C> {}

/// A stripped-down state event, used for previews of rooms the user has been invited to.
#[derive(Clone, Debug, Event)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct StrippedStateEvent<C: PossiblyRedactedStateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This must be a string type, and is often an empty string.
    ///
    /// A state event is keyed by its `(type, state_key)` tuple. Sending another state event with
    /// the same tuple replaces the previous one.
    pub state_key: C::StateKey,

    /// Timestamp on the originating homeserver when this event was sent.
    ///
    /// This field is usually stripped, but some events might include it.
    #[cfg(feature = "unstable-msc4319")]
    #[ruma_event(default)]
    pub origin_server_ts: Option<MilliSecondsSinceUnixEpoch>,

    /// Additional key-value pairs not signed by the homeserver.
    #[cfg(feature = "unstable-msc4319")]
    pub unsigned: Option<Raw<crate::StateUnsigned<C>>>,
}

impl<C: PossiblyRedactedStateEventContent> JsonCastable<JsonObject> for StrippedStateEvent<C> {}

/// A minimal state event, used for creating a new room.
#[derive(Clone, Debug, Event)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct InitialStateEvent<C: StaticStateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This must be a string type, and is often an empty string.
    ///
    /// A state event is keyed by its `(type, state_key)` tuple. Sending another state event with
    /// the same tuple replaces the previous one.
    ///
    /// Defaults to the empty string.
    pub state_key: C::StateKey,
}

impl<C: StaticStateEventContent> InitialStateEvent<C> {
    /// Create a new `InitialStateEvent` for an event type with the given state key.
    ///
    /// For cases where the state key is empty,
    /// [`with_empty_state_key()`](Self::with_empty_state_key) can be used instead.
    pub fn new(state_key: C::StateKey, content: C) -> Self {
        Self { content, state_key }
    }

    /// Create a new `InitialStateEvent` for an event type with an empty state key.
    ///
    /// For cases where the state key is not empty, use [`new()`](Self::new).
    pub fn with_empty_state_key(content: C) -> Self
    where
        C: StaticStateEventContent<StateKey = EmptyStateKey>,
    {
        Self::new(EmptyStateKey, content)
    }

    /// Shorthand for `Raw::new(self).unwrap()`.
    ///
    /// Since none of the content types in Ruma ever return an error in serialization, this will
    /// never panic with `C` being a type from Ruma. However, if you use a custom content type
    /// with a `Serialize` implementation that can error (for example because it contains an
    /// `enum` with one or more variants that use the `#[serde(skip)]` attribute), this method
    /// can panic.
    pub fn to_raw(&self) -> Raw<Self> {
        Raw::new(self).unwrap()
    }

    /// Shorthand for `self.to_raw().cast::<AnyInitialStateEvent>()`.
    ///
    /// Since none of the content types in Ruma ever return an error in serialization, this will
    /// never panic with `C` being a type from Ruma. However, if you use a custom content type
    /// with a `Serialize` implementation that can error (for example because it contains an
    /// `enum` with one or more variants that use the `#[serde(skip)]` attribute), this method
    /// can panic.
    pub fn to_raw_any(&self) -> Raw<AnyInitialStateEvent> {
        self.to_raw().cast()
    }
}

impl<C> Default for InitialStateEvent<C>
where
    C: StaticStateEventContent<StateKey = EmptyStateKey> + Default,
{
    fn default() -> Self {
        Self { content: Default::default(), state_key: EmptyStateKey }
    }
}

impl<C: StaticStateEventContent> Serialize for InitialStateEvent<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("InitialStateEvent", 3)?;
        state.serialize_field("type", &self.content.event_type())?;
        state.serialize_field("content", &self.content)?;
        state.serialize_field("state_key", &self.state_key)?;
        state.end()
    }
}

impl<C: StaticStateEventContent> JsonCastable<JsonObject> for InitialStateEvent<C> {}

/// A redacted state event.
///
/// `RedactedStateEvent` implements the comparison traits using only the `event_id` field, a sorted
/// list would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RedactedStateEvent<C: RedactedStateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique identifier for the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp on the originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The ID of the room associated with this event.
    pub room_id: RoomId,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This must be a string type, and is often an empty string.
    ///
    /// A state event is keyed by its `(type, state_key)` tuple. Sending another state event with
    /// the same tuple replaces the previous one.
    pub state_key: C::StateKey,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RedactedUnsigned,
}

impl<C: RedactedStateEventContent> JsonCastable<RedactedSyncStateEvent<C>>
    for RedactedStateEvent<C>
{
}

impl<C: StaticStateEventContent + RedactContent> JsonCastable<StateEvent<C>>
    for RedactedStateEvent<C::Redacted>
where
    C::Redacted: RedactedStateEventContent,
{
}

impl<C: StaticStateEventContent + RedactContent> JsonCastable<SyncStateEvent<C>>
    for RedactedStateEvent<C::Redacted>
where
    C::Redacted: RedactedStateEventContent,
{
}

impl<C: RedactedStateEventContent> JsonCastable<JsonObject> for RedactedStateEvent<C> {}

/// A redacted state event without a `room_id`.
///
/// `RedactedSyncStateEvent` implements the comparison traits using only the `event_id` field, a
/// sorted list would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RedactedSyncStateEvent<C: RedactedStateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique identifier for the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp on the originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This must be a string type, and is often an empty string.
    ///
    /// A state event is keyed by its `(type, state_key)` tuple. Sending another state event with
    /// the same tuple replaces the previous one.
    pub state_key: C::StateKey,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RedactedUnsigned,
}

impl<C: StaticStateEventContent + RedactContent> JsonCastable<SyncStateEvent<C>>
    for RedactedSyncStateEvent<C::Redacted>
where
    C::Redacted: RedactedStateEventContent,
{
}

impl<C: RedactedStateEventContent> JsonCastable<JsonObject> for RedactedSyncStateEvent<C> {}

/// A possibly-redacted state event.
///
/// `StateEvent` implements the comparison traits using only the `event_id` field, a sorted list
/// would be sorted lexicographically based on the event's `EventId`.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Debug)]
pub enum StateEvent<C: StaticStateEventContent + RedactContent>
where
    C::Redacted: RedactedStateEventContent,
{
    /// Original, unredacted form of the event.
    Original(OriginalStateEvent<C>),

    /// Redacted form of the event with minimal fields.
    Redacted(RedactedStateEvent<C::Redacted>),
}

impl<C: StaticStateEventContent + RedactContent> JsonCastable<SyncStateEvent<C>> for StateEvent<C> where
    C::Redacted: RedactedStateEventContent
{
}

impl<C: StaticStateEventContent + RedactContent>
    JsonCastable<StrippedStateEvent<C::PossiblyRedacted>> for StateEvent<C>
where
    C::Redacted: RedactedStateEventContent,
    C::PossiblyRedacted: PossiblyRedactedStateEventContent,
{
}

impl<C: StaticStateEventContent + RedactContent> JsonCastable<JsonObject> for StateEvent<C> where
    C::Redacted: RedactedStateEventContent
{
}

/// A possibly-redacted state event without a `room_id`.
///
/// `SyncStateEvent` implements the comparison traits using only the `event_id` field, a sorted list
/// would be sorted lexicographically based on the event's `EventId`.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Debug)]
pub enum SyncStateEvent<C: StaticStateEventContent + RedactContent>
where
    C::Redacted: RedactedStateEventContent,
{
    /// Original, unredacted form of the event.
    Original(OriginalSyncStateEvent<C>),

    /// Redacted form of the event with minimal fields.
    Redacted(RedactedSyncStateEvent<C::Redacted>),
}

impl<C: StaticStateEventContent + RedactContent>
    JsonCastable<StrippedStateEvent<C::PossiblyRedacted>> for SyncStateEvent<C>
where
    C::Redacted: RedactedStateEventContent,
    C::PossiblyRedacted: PossiblyRedactedStateEventContent,
{
}

impl<C: StaticStateEventContent + RedactContent> JsonCastable<JsonObject> for SyncStateEvent<C> where
    C::Redacted: RedactedStateEventContent
{
}

/// An event sent using send-to-device messaging.
#[derive(Clone, Debug, Event)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct ToDeviceEvent<C: ToDeviceEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,
}

impl<C: ToDeviceEventContent> ToDeviceEvent<C> {
    /// Construct a new `ToDeviceEvent` with the given content and sender.
    pub fn new(sender: UserId, content: C) -> Self {
        Self { content, sender }
    }
}

impl<C: ToDeviceEventContent> Serialize for ToDeviceEvent<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("ToDeviceEvent", 3)?;
        state.serialize_field("type", &self.content.event_type())?;
        state.serialize_field("content", &self.content)?;
        state.serialize_field("sender", &self.sender)?;
        state.end()
    }
}

impl<C: ToDeviceEventContent> JsonCastable<JsonObject> for ToDeviceEvent<C> {}

/// The decrypted payload of an `m.olm.v1.curve25519-aes-sha2` event.
#[derive(Clone, Debug, Event)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct DecryptedOlmV1Event<C: MessageLikeEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// The fully-qualified ID of the intended recipient this event.
    pub recipient: UserId,

    /// The recipient's ed25519 key.
    pub recipient_keys: OlmV1Keys,

    /// The sender's ed25519 key.
    pub keys: OlmV1Keys,

    /// The sender's device keys.
    pub sender_device_keys: Option<Raw<DeviceKeys>>,
}

/// Public keys used for an `m.olm.v1.curve25519-aes-sha2` event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct OlmV1Keys {
    /// An ed25519 key.
    pub ed25519: String,
}

impl OlmV1Keys {
    /// Construct a new `OlmV1Keys` with the given ed25519 key.
    pub fn new(ed25519: String) -> Self {
        Self { ed25519 }
    }
}

/// The decrypted payload of an `m.megolm.v1.aes-sha2` event.
#[derive(Clone, Debug, Event)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct DecryptedMegolmV1Event<C: MessageLikeEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The ID of the room associated with the event.
    pub room_id: RoomId,
}

/// A possibly-redacted state event content.
///
/// A non-redacted content also contains the `prev_content` from the unsigned event data.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Debug)]
pub enum FullStateEventContent<C: StaticStateEventContent + RedactContent> {
    /// Original, unredacted content of the event.
    Original {
        /// Current content of the room state.
        content: C,

        /// Previous content of the room state.
        prev_content: Option<C::PossiblyRedacted>,
    },

    /// Redacted content of the event.
    Redacted(C::Redacted),
}

impl<C: StaticStateEventContent + RedactContent> FullStateEventContent<C>
where
    C::Redacted: RedactedStateEventContent,
{
    /// Get the event’s type, like `m.room.create`.
    pub fn event_type(&self) -> StateEventType {
        match self {
            Self::Original { content, .. } => content.event_type(),
            Self::Redacted(content) => content.event_type(),
        }
    }

    /// Transform `self` into a redacted form (removing most or all fields) according to the spec.
    ///
    /// If `self` is already [`Redacted`](Self::Redacted), return the inner data unmodified.
    ///
    /// A small number of events have room-version specific redaction behavior, so a
    /// [`RedactionRules`] has to be specified.
    pub fn redact(self, rules: &RedactionRules) -> C::Redacted {
        match self {
            FullStateEventContent::Original { content, .. } => content.redact(rules),
            FullStateEventContent::Redacted(content) => content,
        }
    }
}

macro_rules! impl_possibly_redacted_event {
    (
        $ty:ident ( $content_trait:ident, $redacted_content_trait:ident, $event_type:ident )
        $( where C::Redacted: $trait:ident<StateKey = C::StateKey>, )?
        { $($extra:tt)* }
    ) => {
        impl<C> $ty<C>
        where
            C: $content_trait + RedactContent,
            C::Redacted: $redacted_content_trait,
            $( C::Redacted: $trait<StateKey = C::StateKey>, )?
        {
            /// Returns the `type` of this event.
            pub fn event_type(&self) -> $event_type {
                match self {
                    Self::Original(ev) => ev.content.event_type(),
                    Self::Redacted(ev) => ev.content.event_type(),
                }
            }

            /// Returns this event's `event_id` field.
            pub fn event_id(&self) -> &EventId {
                match self {
                    Self::Original(ev) => &ev.event_id,
                    Self::Redacted(ev) => &ev.event_id,
                }
            }

            /// Returns this event's `sender` field.
            pub fn sender(&self) -> &UserId {
                match self {
                    Self::Original(ev) => &ev.sender,
                    Self::Redacted(ev) => &ev.sender,
                }
            }

            /// Returns this event's `origin_server_ts` field.
            pub fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch {
                match self {
                    Self::Original(ev) => ev.origin_server_ts,
                    Self::Redacted(ev) => ev.origin_server_ts,
                }
            }

            // So the room_id method can be in the same impl block, in rustdoc
            $($extra)*
        }

        impl<'de, C> Deserialize<'de> for $ty<C>
        where
            C: $content_trait + EventContentFromType + RedactContent,
            C::Redacted: $redacted_content_trait + EventContentFromType,
            $( C::Redacted: $trait<StateKey = C::StateKey>, )?
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let json = Box::<RawJsonValue>::deserialize(deserializer)?;
                let RedactionDeHelper { unsigned } = from_raw_json_value(&json)?;

                if unsigned.and_then(|u| u.redacted_because).is_some() {
                    Ok(Self::Redacted(from_raw_json_value(&json)?))
                } else {
                    Ok(Self::Original(from_raw_json_value(&json)?))
                }
            }
        }
    }
}

impl_possibly_redacted_event!(
    MessageLikeEvent(
        MessageLikeEventContent, RedactedMessageLikeEventContent, MessageLikeEventType
    ) {
        /// Returns this event's `room_id` field.
        pub fn room_id(&self) -> &RoomId {
            match self {
                Self::Original(ev) => &ev.room_id,
                Self::Redacted(ev) => &ev.room_id,
            }
        }

        /// Get the inner `OriginalMessageLikeEvent` if this is an unredacted event.
        pub fn as_original(&self) -> Option<&OriginalMessageLikeEvent<C>> {
            as_variant!(self, Self::Original)
        }
    }
);

impl_possibly_redacted_event!(
    SyncMessageLikeEvent(
        MessageLikeEventContent, RedactedMessageLikeEventContent, MessageLikeEventType
    ) {
        /// Get the inner `OriginalSyncMessageLikeEvent` if this is an unredacted event.
        pub fn as_original(&self) -> Option<&OriginalSyncMessageLikeEvent<C>> {
            as_variant!(self, Self::Original)
        }

        /// Convert this sync event into a full event (one with a `room_id` field).
        pub fn into_full_event(self, room_id: RoomId) -> MessageLikeEvent<C> {
            match self {
                Self::Original(ev) => MessageLikeEvent::Original(ev.into_full_event(room_id)),
                Self::Redacted(ev) => MessageLikeEvent::Redacted(ev.into_full_event(room_id)),
            }
        }
    }
);

impl_possibly_redacted_event!(
    StateEvent(StaticStateEventContent, RedactedStateEventContent, StateEventType)
    where
        C::Redacted: RedactedStateEventContent<StateKey = C::StateKey>,
    {
        /// Returns this event's `room_id` field.
        pub fn room_id(&self) -> &RoomId {
            match self {
                Self::Original(ev) => &ev.room_id,
                Self::Redacted(ev) => &ev.room_id,
            }
        }

        /// Returns this event's `state_key` field.
        pub fn state_key(&self) -> &C::StateKey {
            match self {
                Self::Original(ev) => &ev.state_key,
                Self::Redacted(ev) => &ev.state_key,
            }
        }

        /// Get the inner `OriginalStateEvent` if this is an unredacted event.
        pub fn as_original(&self) -> Option<&OriginalStateEvent<C>> {
            as_variant!(self, Self::Original)
        }
    }
);

impl_possibly_redacted_event!(
    SyncStateEvent(StaticStateEventContent, RedactedStateEventContent, StateEventType)
    where
        C::Redacted: RedactedStateEventContent<StateKey = C::StateKey>,
    {
        /// Returns this event's `state_key` field.
        pub fn state_key(&self) -> &C::StateKey {
            match self {
                Self::Original(ev) => &ev.state_key,
                Self::Redacted(ev) => &ev.state_key,
            }
        }

        /// Get the inner `OriginalSyncStateEvent` if this is an unredacted event.
        pub fn as_original(&self) -> Option<&OriginalSyncStateEvent<C>> {
            as_variant!(self, Self::Original)
        }

        /// Convert this sync event into a full event (one with a `room_id` field).
        pub fn into_full_event(self, room_id: RoomId) -> StateEvent<C> {
            match self {
                Self::Original(ev) => StateEvent::Original(ev.into_full_event(room_id)),
                Self::Redacted(ev) => StateEvent::Redacted(ev.into_full_event(room_id)),
            }
        }
    }
);

macro_rules! impl_sync_from_full {
    ($ty:ident, $full:ident, $content_trait:ident, $redacted_content_trait: ident) => {
        impl<C> From<$full<C>> for $ty<C>
        where
            C: $content_trait + RedactContent,
            C::Redacted: $redacted_content_trait,
        {
            fn from(full: $full<C>) -> Self {
                match full {
                    $full::Original(ev) => Self::Original(ev.into()),
                    $full::Redacted(ev) => Self::Redacted(ev.into()),
                }
            }
        }
    };
}

impl_sync_from_full!(
    SyncMessageLikeEvent,
    MessageLikeEvent,
    MessageLikeEventContent,
    RedactedMessageLikeEventContent
);
impl_sync_from_full!(
    SyncStateEvent,
    StateEvent,
    StaticStateEventContent,
    RedactedStateEventContent
);
