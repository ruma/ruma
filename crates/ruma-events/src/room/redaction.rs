//! Types for the *m.room.redaction* event.

use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events_macros::{Event, EventContent};
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{Deserialize, Serialize};

use crate::Unsigned;

/// Redaction event.
#[derive(Clone, Debug, Event)]
pub struct RedactionEvent {
    /// Data specific to the event type.
    pub content: RedactionEventContent,

    /// The ID of the event that was redacted.
    pub redacts: EventId,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// The ID of the room associated with this event.
    pub room_id: RoomId,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: Unsigned,
}

/// Redaction event without a `room_id`.
#[derive(Clone, Debug, Event)]
pub struct SyncRedactionEvent {
    /// Data specific to the event type.
    pub content: RedactionEventContent,

    /// The ID of the event that was redacted.
    pub redacts: EventId,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// The fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: Unsigned,
}

/// A redaction of an event.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.redaction", kind = Message)]
pub struct RedactionEventContent {
    /// The reason for the redaction, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl RedactionEventContent {
    /// Creates an empty `RedactionEventContent`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `RedactionEventContent` with the given reason.
    pub fn with_reason(reason: String) -> Self {
        Self { reason: Some(reason) }
    }
}
