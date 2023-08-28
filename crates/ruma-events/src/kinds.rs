#![allow(clippy::exhaustive_structs)]

use as_variant::as_variant;
use ruma_common::{
    serde::{from_raw_json_value, Raw},
    EventId, MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId, OwnedUserId, RoomId,
    RoomVersionId, UserId,
};
use ruma_macros::Event;
use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize};
use serde_json::value::RawValue as RawJsonValue;

use super::{
    AnyInitialStateEvent, EmptyStateKey, EphemeralRoomEventContent, EventContent,
    EventContentFromType, GlobalAccountDataEventContent, MessageLikeEventContent,
    MessageLikeEventType, MessageLikeUnsigned, PossiblyRedactedStateEventContent, RedactContent,
    RedactedMessageLikeEventContent, RedactedStateEventContent, RedactedUnsigned,
    RedactionDeHelper, RoomAccountDataEventContent, StateEventType, StaticStateEventContent,
    ToDeviceEventContent,
};

/// A global account data event.
#[derive(Clone, Debug, Event)]
pub struct GlobalAccountDataEvent<C: GlobalAccountDataEventContent> {
    /// Data specific to the event type.
    pub content: C,
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

/// A room account data event.
#[derive(Clone, Debug, Event)]
pub struct RoomAccountDataEvent<C: RoomAccountDataEventContent> {
    /// Data specific to the event type.
    pub content: C,
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

/// An ephemeral room event.
#[derive(Clone, Debug, Event)]
pub struct EphemeralRoomEvent<C: EphemeralRoomEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The ID of the room associated with this event.
    pub room_id: OwnedRoomId,
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

/// An ephemeral room event without a `room_id`.
#[derive(Clone, Debug, Event)]
pub struct SyncEphemeralRoomEvent<C: EphemeralRoomEventContent> {
    /// Data specific to the event type.
    pub content: C,
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

/// An unredacted message-like event.
///
/// `OriginalMessageLikeEvent` implements the comparison traits using only the `event_id` field, a
/// sorted list would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct OriginalMessageLikeEvent<C: MessageLikeEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: OwnedEventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The ID of the room associated with this event.
    pub room_id: OwnedRoomId,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: MessageLikeUnsigned<C>,
}

/// An unredacted message-like event without a `room_id`.
///
/// `OriginalSyncMessageLikeEvent` implements the comparison traits using only the `event_id` field,
/// a sorted list would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct OriginalSyncMessageLikeEvent<C: MessageLikeEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: OwnedEventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
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

/// A redacted message-like event.
///
/// `RedactedMessageLikeEvent` implements the comparison traits using only the `event_id` field, a
/// sorted list would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct RedactedMessageLikeEvent<C: RedactedMessageLikeEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: OwnedEventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The ID of the room associated with this event.
    pub room_id: OwnedRoomId,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RedactedUnsigned,
}

/// A redacted message-like event without a `room_id`.
///
/// `RedactedSyncMessageLikeEvent` implements the comparison traits using only the `event_id` field,
/// a sorted list would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct RedactedSyncMessageLikeEvent<C: RedactedMessageLikeEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: OwnedEventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RedactedUnsigned,
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

/// An unredacted state event.
///
/// `OriginalStateEvent` implements the comparison traits using only the `event_id` field, a sorted
/// list would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct OriginalStateEvent<C: StaticStateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: OwnedEventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The ID of the room associated with this event.
    pub room_id: OwnedRoomId,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show which user the event
    /// affects.
    pub state_key: C::StateKey,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: C::Unsigned,
}

/// An unredacted state event without a `room_id`.
///
/// `OriginalSyncStateEvent` implements the comparison traits using only the `event_id` field, a
/// sorted list would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct OriginalSyncStateEvent<C: StaticStateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: OwnedEventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show which user the event
    /// affects.
    pub state_key: C::StateKey,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: C::Unsigned,
}

/// A stripped-down state event, used for previews of rooms the user has been invited to.
#[derive(Clone, Debug, Event)]
pub struct StrippedStateEvent<C: PossiblyRedactedStateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show which user the event
    /// affects.
    pub state_key: C::StateKey,
}

/// A minimal state event, used for creating a new room.
#[derive(Clone, Debug, Event)]
pub struct InitialStateEvent<C: StaticStateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show which user the event
    /// affects.
    ///
    /// Defaults to the empty string.
    pub state_key: C::StateKey,
}

impl<C: StaticStateEventContent> InitialStateEvent<C> {
    /// Create a new `InitialStateEvent` for an event type with an empty state key.
    ///
    /// For cases where the state key is not empty, use a struct literal to create the event.
    pub fn new(content: C) -> Self
    where
        C: StaticStateEventContent<StateKey = EmptyStateKey>,
    {
        Self { content, state_key: EmptyStateKey }
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

/// A redacted state event.
///
/// `RedactedStateEvent` implements the comparison traits using only the `event_id` field, a sorted
/// list would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct RedactedStateEvent<C: RedactedStateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: OwnedEventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The ID of the room associated with this event.
    pub room_id: OwnedRoomId,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show which user the event
    /// affects.
    pub state_key: C::StateKey,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RedactedUnsigned,
}

/// A redacted state event without a `room_id`.
///
/// `RedactedSyncStateEvent` implements the comparison traits using only the `event_id` field, a
/// sorted list would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct RedactedSyncStateEvent<C: RedactedStateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: OwnedEventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show which user the event
    /// affects.
    pub state_key: C::StateKey,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RedactedUnsigned,
}

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

/// An event sent using send-to-device messaging.
#[derive(Clone, Debug, Event)]
pub struct ToDeviceEvent<C: ToDeviceEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,
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

/// The decrypted payload of an `m.olm.v1.curve25519-aes-sha2` event.
#[derive(Clone, Debug, Event)]
pub struct DecryptedOlmV1Event<C: MessageLikeEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,

    /// The fully-qualified ID of the intended recipient this event.
    pub recipient: OwnedUserId,

    /// The recipient's ed25519 key.
    pub recipient_keys: OlmV1Keys,

    /// The sender's ed25519 key.
    pub keys: OlmV1Keys,
}

/// Public keys used for an `m.olm.v1.curve25519-aes-sha2` event.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OlmV1Keys {
    /// An ed25519 key.
    pub ed25519: String,
}

/// The decrypted payload of an `m.megolm.v1.aes-sha2` event.
#[derive(Clone, Debug, Event)]
pub struct DecryptedMegolmV1Event<C: MessageLikeEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The ID of the room associated with the event.
    pub room_id: OwnedRoomId,
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
    /// A small number of events have room-version specific redaction behavior, so a version has to
    /// be specified.
    pub fn redact(self, version: &RoomVersionId) -> C::Redacted {
        match self {
            FullStateEventContent::Original { content, .. } => content.redact(version),
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
        pub fn into_full_event(self, room_id: OwnedRoomId) -> MessageLikeEvent<C> {
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
        pub fn into_full_event(self, room_id: OwnedRoomId) -> StateEvent<C> {
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
