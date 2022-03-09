//! Types for the [`m.room.redaction`] event.
//!
//! [`m.room.redaction`]: https://spec.matrix.org/v1.2/client-server-api/#mroomredaction

use ruma_macros::{Event, EventContent};
use serde::{Deserialize, Serialize};

use crate::{
    events::{Redact, RedactContent, RedactedUnsigned, Unsigned},
    EventId, MilliSecondsSinceUnixEpoch, RoomId, UserId,
};

/// Redaction event.
#[derive(Clone, Debug, Event)]
#[allow(clippy::exhaustive_structs)]
pub struct RoomRedactionEvent {
    /// Data specific to the event type.
    pub content: RoomRedactionEventContent,

    /// The ID of the event that was redacted.
    pub redacts: Box<EventId>,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: Box<EventId>,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: Box<UserId>,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The ID of the room associated with this event.
    pub room_id: Box<RoomId>,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: Unsigned,
}

impl Redact for RoomRedactionEvent {
    type Redacted = RedactedRoomRedactionEvent;

    fn redact(
        self,
        redaction: SyncRoomRedactionEvent,
        version: &crate::RoomVersionId,
    ) -> Self::Redacted {
        RedactedRoomRedactionEvent {
            content: self.content.redact(version),
            // There is no released room version where this isn't redacted yet
            redacts: None,
            event_id: self.event_id,
            sender: self.sender,
            origin_server_ts: self.origin_server_ts,
            room_id: self.room_id,
            unsigned: RedactedUnsigned::new_because(Box::new(redaction)),
        }
    }
}

/// Redacted redaction event.
#[derive(Clone, Debug, Event)]
#[allow(clippy::exhaustive_structs)]
pub struct RedactedRoomRedactionEvent {
    /// Data specific to the event type.
    pub content: RedactedRoomRedactionEventContent,

    /// The ID of the event that was redacted.
    pub redacts: Option<Box<EventId>>,

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

/// Redaction event without a `room_id`.
#[derive(Clone, Debug, Event)]
#[allow(clippy::exhaustive_structs)]
pub struct SyncRoomRedactionEvent {
    /// Data specific to the event type.
    pub content: RoomRedactionEventContent,

    /// The ID of the event that was redacted.
    pub redacts: Box<EventId>,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: Box<EventId>,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: Box<UserId>,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: Unsigned,
}

impl Redact for SyncRoomRedactionEvent {
    type Redacted = RedactedSyncRoomRedactionEvent;

    fn redact(
        self,
        redaction: SyncRoomRedactionEvent,
        version: &crate::RoomVersionId,
    ) -> Self::Redacted {
        RedactedSyncRoomRedactionEvent {
            content: self.content.redact(version),
            // There is no released room version where this isn't redacted yet
            redacts: None,
            event_id: self.event_id,
            sender: self.sender,
            origin_server_ts: self.origin_server_ts,
            unsigned: RedactedUnsigned::new_because(Box::new(redaction)),
        }
    }
}

/// Redacted redaction event without a `room_id`.
#[derive(Clone, Debug, Event)]
#[allow(clippy::exhaustive_structs)]
pub struct RedactedSyncRoomRedactionEvent {
    /// Data specific to the event type.
    pub content: RedactedRoomRedactionEventContent,

    /// The ID of the event that was redacted.
    pub redacts: Option<Box<EventId>>,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: Box<EventId>,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: Box<UserId>,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: RedactedUnsigned,
}

/// A redaction of an event.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.redaction", kind = MessageLike)]
pub struct RoomRedactionEventContent {
    /// The reason for the redaction, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl RoomRedactionEventContent {
    /// Creates an empty `RoomRedactionEventContent`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `RoomRedactionEventContent` with the given reason.
    pub fn with_reason(reason: String) -> Self {
        Self { reason: Some(reason) }
    }
}
