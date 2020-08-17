use std::time::SystemTime;

use ruma_events_macros::Event;
use ruma_identifiers::{EventId, RoomId, UserId};

use crate::{
    BasicEventContent, EphemeralRoomEventContent, EventContent, MessageEventContent,
    RedactedMessageEventContent, RedactedStateEventContent, RedactedSyncUnsigned, RedactedUnsigned,
    StateEventContent, Unsigned,
};

/// A basic event – one that consists only of it's type and the `content` object.
#[derive(Clone, Debug, Event)]
pub struct BasicEvent<C: BasicEventContent> {
    /// Data specific to the event type.
    pub content: C,
}

/// An ephemeral room event.
#[derive(Clone, Debug, Event)]
pub struct EphemeralRoomEvent<C: EphemeralRoomEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The ID of the room associated with this event.
    pub room_id: RoomId,
}

/// An ephemeral room event without a `room_id`.
#[derive(Clone, Debug, Event)]
pub struct SyncEphemeralRoomEvent<C: EphemeralRoomEventContent> {
    /// Data specific to the event type.
    pub content: C,
}

/// A message event.
///
/// `MessageEvent` implements the comparison traits using only
/// the `event_id` field, a sorted list would be sorted lexicographically based on
/// the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct MessageEvent<C: MessageEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// The ID of the room associated with this event.
    pub room_id: RoomId,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: Unsigned,
}

/// A message event without a `room_id`.
///
/// `SyncMessageEvent` implements the comparison traits using only
/// the `event_id` field, a sorted list would be sorted lexicographically based on
/// the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct SyncMessageEvent<C: MessageEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: Unsigned,
}

/// A redacted message event.
///
/// `RedactedMessageEvent` implements the comparison traits using only
/// the `event_id` field, a sorted list would be sorted lexicographically based on
/// the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct RedactedMessageEvent<C: RedactedMessageEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// The ID of the room associated with this event.
    pub room_id: RoomId,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RedactedUnsigned,
}

/// A redacted message event without a `room_id`.
///
/// `RedactedSyncMessageEvent` implements the comparison traits using only
/// the `event_id` field, a sorted list would be sorted lexicographically based on
/// the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct RedactedSyncMessageEvent<C: RedactedMessageEventContent> {
    /// Data specific to the event type.
    // #[serde(default, skip_serializing_if = "is_zst")]
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RedactedSyncUnsigned,
}

/// A state event.
///
/// `StateEvent` implements the comparison traits using only
/// the `event_id` field, a sorted list would be sorted lexicographically based on
/// the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct StateEvent<C: StateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// The ID of the room associated with this event.
    pub room_id: RoomId,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show
    /// which user the event affects.
    pub state_key: String,

    /// Optional previous content for this event.
    pub prev_content: Option<C>,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: Unsigned,
}

/// A state event without a `room_id`.
///
/// `SyncStateEvent` implements the comparison traits using only
/// the `event_id` field, a sorted list would be sorted lexicographically based on
/// the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct SyncStateEvent<C: StateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show
    /// which user the event affects.
    pub state_key: String,

    /// Optional previous content for this event.
    pub prev_content: Option<C>,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: Unsigned,
}

/// A stripped-down state event, used for previews of rooms the user has been
/// invited to.
#[derive(Clone, Debug, Event)]
pub struct StrippedStateEvent<C: StateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show
    /// which user the event affects.
    pub state_key: String,
}

/// A minimal state event, used for creating a new room.
#[derive(Clone, Debug, Event)]
pub struct InitialStateEvent<C: StateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show
    /// which user the event affects.
    pub state_key: String,
}

/// A redacted state event.
///
/// `RedactedStateEvent` implements the comparison traits using only
/// the `event_id` field, a sorted list would be sorted lexicographically based on
/// the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct RedactedStateEvent<C: RedactedStateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// The ID of the room associated with this event.
    pub room_id: RoomId,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show
    /// which user the event affects.
    pub state_key: String,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RedactedUnsigned,
}

/// A redacted state event without a `room_id`.
///
/// `RedactedSyncStateEvent` implements the comparison traits using only
/// the `event_id` field, a sorted list would be sorted lexicographically based on
/// the event's `EventId`.
#[derive(Clone, Debug, Event)]
pub struct RedactedSyncStateEvent<C: RedactedStateEventContent> {
    /// Data specific to the event type.
    // #[serde(default, skip_serializing_if = "is_zst")]
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show
    /// which user the event affects.
    pub state_key: String,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RedactedSyncUnsigned,
}

/// A stripped-down redacted state event.
#[derive(Clone, Debug, Event)]
pub struct RedactedStrippedStateEvent<C: RedactedStateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show
    /// which user the event affects.
    pub state_key: String,
}

/// An event sent using send-to-device messaging.
#[derive(Clone, Debug, Event)]
pub struct ToDeviceEvent<C: EventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,
}
