#![allow(clippy::exhaustive_structs)]

use ruma_macros::Event;
use serde::{Deserialize, Serialize};

use super::{
    EphemeralRoomEventType, EventContent, GlobalAccountDataEventType, MessageLikeEventType,
    MessageLikeUnsigned, RedactedEventContent, RedactedUnsigned, RoomAccountDataEventType,
    StateEventType, StateUnsigned, ToDeviceEventType,
};
use crate::{EventId, MilliSecondsSinceUnixEpoch, RoomId, UserId};

/// A global account data event.
#[derive(Clone, Debug, Event)]
pub struct GlobalAccountDataEvent<C: EventContent<EventType = GlobalAccountDataEventType>> {
    /// Data specific to the event type.
    pub content: C,
}

/// A room account data event.
#[derive(Clone, Debug, Event)]
pub struct RoomAccountDataEvent<C: EventContent<EventType = RoomAccountDataEventType>> {
    /// Data specific to the event type.
    pub content: C,
}

/// An ephemeral room event.
#[derive(Clone, Debug, Event)]
pub struct EphemeralRoomEvent<C: EventContent<EventType = EphemeralRoomEventType>> {
    /// Data specific to the event type.
    pub content: C,

    /// The ID of the room associated with this event.
    pub room_id: Box<RoomId>,
}

/// An ephemeral room event without a `room_id`.
#[derive(Clone, Debug, Event)]
pub struct SyncEphemeralRoomEvent<C: EventContent<EventType = EphemeralRoomEventType>> {
    /// Data specific to the event type.
    pub content: C,
}

/// A message-like event.
///
/// `MessageLikeEvent` implements the comparison traits using only the `event_id` field, a sorted
/// list would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct MessageLikeEvent<C: EventContent<EventType = MessageLikeEventType>> {
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

/// A message-like event without a `room_id`.
///
/// `SyncMessageLikeEvent` implements the comparison traits using only the `event_id` field, a
/// sorted list would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct SyncMessageLikeEvent<C: EventContent<EventType = MessageLikeEventType>> {
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
pub struct RedactedMessageLikeEvent<
    C: EventContent<EventType = MessageLikeEventType> + RedactedEventContent,
> {
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
pub struct RedactedSyncMessageLikeEvent<
    C: EventContent<EventType = MessageLikeEventType> + RedactedEventContent,
> {
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

/// A state event.
///
/// `StateEvent` implements the comparison traits using only the `event_id` field, a sorted list
/// would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct StateEvent<C: EventContent<EventType = StateEventType>> {
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

/// A state event without a `room_id`.
///
/// `SyncStateEvent` implements the comparison traits using only the `event_id` field, a sorted list
/// would be sorted lexicographically based on the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct SyncStateEvent<C: EventContent<EventType = StateEventType>> {
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
pub struct StrippedStateEvent<C: EventContent<EventType = StateEventType>> {
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
pub struct InitialStateEvent<C: EventContent<EventType = StateEventType>> {
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
pub struct RedactedStateEvent<C: EventContent<EventType = StateEventType> + RedactedEventContent> {
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
pub struct RedactedSyncStateEvent<
    C: EventContent<EventType = StateEventType> + RedactedEventContent,
> {
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

/// An event sent using send-to-device messaging.
#[derive(Clone, Debug, Event)]
pub struct ToDeviceEvent<C: EventContent<EventType = ToDeviceEventType>> {
    /// Data specific to the event type.
    pub content: C,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: Box<UserId>,
}

/// The decrypted payload of an `m.olm.v1.curve25519-aes-sha2` event.
#[derive(Clone, Debug, Event)]
pub struct DecryptedOlmV1Event<C: EventContent<EventType = MessageLikeEventType>> {
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
pub struct DecryptedMegolmV1Event<C: EventContent<EventType = MessageLikeEventType>> {
    /// Data specific to the event type.
    pub content: C,

    /// The ID of the room associated with the event.
    pub room_id: Box<RoomId>,
}
