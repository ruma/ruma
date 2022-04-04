#![allow(clippy::exhaustive_structs)]

use ruma_macros::Event;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::value::RawValue as RawJsonValue;

use super::{
    room::redaction::SyncRoomRedactionEvent, EphemeralRoomEventContent, EventContent,
    GlobalAccountDataEventContent, MessageLikeEventContent, MessageLikeEventType,
    MessageLikeUnsigned, Redact, RedactContent, RedactedEventContent,
    RedactedMessageLikeEventContent, RedactedStateEventContent, RedactedUnsigned,
    RedactionDeHelper, RoomAccountDataEventContent, StateEventContent, StateEventType,
    StateUnsigned, ToDeviceEventContent,
};
use crate::{
    serde::from_raw_json_value, EventId, MilliSecondsSinceUnixEpoch, RoomId, RoomVersionId, UserId,
};

/// A global account data event.
#[derive(Clone, Debug, Event)]
pub struct GlobalAccountDataEvent<C: GlobalAccountDataEventContent> {
    /// Data specific to the event type.
    pub content: C,
}

/// A room account data event.
#[derive(Clone, Debug, Event)]
pub struct RoomAccountDataEvent<C: RoomAccountDataEventContent> {
    /// Data specific to the event type.
    pub content: C,
}

/// An ephemeral room event.
#[derive(Clone, Debug, Event)]
pub struct EphemeralRoomEvent<C: EphemeralRoomEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The ID of the room associated with this event.
    pub room_id: Box<RoomId>,
}

/// An ephemeral room event without a `room_id`.
#[derive(Clone, Debug, Event)]
pub struct SyncEphemeralRoomEvent<C: EphemeralRoomEventContent> {
    /// Data specific to the event type.
    pub content: C,
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
    pub event_id: Box<EventId>,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: Box<UserId>,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The ID of the room associated with this event.
    pub room_id: Box<RoomId>,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: MessageLikeUnsigned,
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
    pub event_id: Box<EventId>,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: Box<UserId>,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: MessageLikeUnsigned,
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
    pub event_id: Box<EventId>,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: Box<UserId>,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The ID of the room associated with this event.
    pub room_id: Box<RoomId>,

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
    pub event_id: Box<EventId>,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: Box<UserId>,

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
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum MessageLikeEvent<C: MessageLikeEventContent + RedactContent>
where
    C::Redacted: MessageLikeEventContent + RedactedEventContent,
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
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum SyncMessageLikeEvent<C: MessageLikeEventContent + RedactContent>
where
    C::Redacted: MessageLikeEventContent + RedactedEventContent,
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
pub struct OriginalStateEvent<C: StateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: Box<EventId>,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: Box<UserId>,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The ID of the room associated with this event.
    pub room_id: Box<RoomId>,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show which user the event
    /// affects.
    pub state_key: String,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: StateUnsigned<C>,
}

/// An unredacted state event without a `room_id`.
///
/// `OriginalSyncStateEvent` implements the comparison traits using only the `event_id` field, a
/// sorted list would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct OriginalSyncStateEvent<C: StateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: Box<EventId>,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: Box<UserId>,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show which user the event
    /// affects.
    pub state_key: String,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: StateUnsigned<C>,
}

/// A stripped-down state event, used for previews of rooms the user has been invited to.
#[derive(Clone, Debug, Event)]
pub struct StrippedStateEvent<C: StateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: Box<UserId>,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show which user the event
    /// affects.
    pub state_key: String,
}

/// A minimal state event, used for creating a new room.
#[derive(Clone, Debug, Event)]
pub struct InitialStateEvent<C: StateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show which user the event
    /// affects.
    ///
    /// Defaults to the empty string.
    #[ruma_event(default)]
    pub state_key: String,
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
    pub event_id: Box<EventId>,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: Box<UserId>,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The ID of the room associated with this event.
    pub room_id: Box<RoomId>,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show which user the event
    /// affects.
    pub state_key: String,

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
    pub event_id: Box<EventId>,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: Box<UserId>,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show which user the event
    /// affects.
    pub state_key: String,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RedactedUnsigned,
}

/// A possibly-redacted state event.
///
/// `StateEvent` implements the comparison traits using only the `event_id` field, a sorted list
/// would be sorted lexicographically based on the event's `EventId`.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum StateEvent<C: StateEventContent + RedactContent>
where
    C::Redacted: StateEventContent + RedactedEventContent,
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
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum SyncStateEvent<C: StateEventContent + RedactContent>
where
    C::Redacted: StateEventContent + RedactedEventContent,
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
    pub sender: Box<UserId>,
}

/// The decrypted payload of an `m.olm.v1.curve25519-aes-sha2` event.
#[derive(Clone, Debug, Event)]
pub struct DecryptedOlmV1Event<C: MessageLikeEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: Box<UserId>,

    /// The fully-qualified ID of the intended recipient this event.
    pub recipient: Box<UserId>,

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
    pub room_id: Box<RoomId>,
}

macro_rules! impl_possibly_redacted_event {
    ($ty:ident ( $content_trait:ident, $event_type:ident ) { $($extra:tt)* }) => {
        impl<C> $ty<C>
        where
            C: $content_trait + RedactContent,
            C::Redacted: $content_trait + RedactedEventContent,
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
            pub fn origin_server_ts(&self) -> &MilliSecondsSinceUnixEpoch {
                match self {
                    Self::Original(ev) => &ev.origin_server_ts,
                    Self::Redacted(ev) => &ev.origin_server_ts,
                }
            }

            // So the room_id method can be in the same impl block, in rustdoc
            $($extra)*
        }

        impl<C> Redact for $ty<C>
        where
            C: $content_trait + RedactContent,
            C::Redacted: $content_trait + RedactedEventContent,
        {
            type Redacted = Self;

            fn redact(self, redaction: SyncRoomRedactionEvent, version: &RoomVersionId) -> Self {
                match self {
                    Self::Original(ev) => Self::Redacted(ev.redact(redaction, version)),
                    Self::Redacted(ev) => Self::Redacted(ev),
                }
            }
        }

        impl<'de, C> Deserialize<'de> for $ty<C>
        where
            C: $content_trait + RedactContent,
            C::Redacted: $content_trait + RedactedEventContent,
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

impl_possibly_redacted_event!(MessageLikeEvent(MessageLikeEventContent, MessageLikeEventType) {
    /// Returns this event's `room_id` field.
    pub fn room_id(&self) -> &RoomId {
        match self {
            Self::Original(ev) => &ev.room_id,
            Self::Redacted(ev) => &ev.room_id,
        }
    }
});

impl_possibly_redacted_event!(SyncMessageLikeEvent(MessageLikeEventContent, MessageLikeEventType) {
    /// Convert this sync event into a full event (one with a `room_id` field).
    pub fn into_full_event(self, room_id: Box<RoomId>) -> MessageLikeEvent<C> {
        match self {
            Self::Original(ev) => MessageLikeEvent::Original(ev.into_full_event(room_id)),
            Self::Redacted(ev) => MessageLikeEvent::Redacted(ev.into_full_event(room_id)),
        }
    }
});

impl_possibly_redacted_event!(StateEvent(StateEventContent, StateEventType) {
    /// Returns this event's `room_id` field.
    pub fn room_id(&self) -> &RoomId {
        match self {
            Self::Original(ev) => &ev.room_id,
            Self::Redacted(ev) => &ev.room_id,
        }
    }

    /// Returns this event's `state_key` field.
    pub fn state_key(&self) -> &str {
        match self {
            Self::Original(ev) => &ev.state_key,
            Self::Redacted(ev) => &ev.state_key,
        }
    }
});

impl_possibly_redacted_event!(SyncStateEvent(StateEventContent, StateEventType) {
    /// Returns this event's `state_key` field.
    pub fn state_key(&self) -> &str {
        match self {
            Self::Original(ev) => &ev.state_key,
            Self::Redacted(ev) => &ev.state_key,
        }
    }

    /// Convert this sync event into a full event (one with a `room_id` field).
    pub fn into_full_event(self, room_id: Box<RoomId>) -> StateEvent<C> {
        match self {
            Self::Original(ev) => StateEvent::Original(ev.into_full_event(room_id)),
            Self::Redacted(ev) => StateEvent::Redacted(ev.into_full_event(room_id)),
        }
    }
});

macro_rules! impl_sync_from_full {
    ($ty:ident, $full:ident, $content_trait:ident) => {
        impl<C> From<$full<C>> for $ty<C>
        where
            C: $content_trait + RedactContent,
            C::Redacted: $content_trait + RedactedEventContent,
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

impl_sync_from_full!(SyncMessageLikeEvent, MessageLikeEvent, MessageLikeEventContent);
impl_sync_from_full!(SyncStateEvent, StateEvent, StateEventContent);
