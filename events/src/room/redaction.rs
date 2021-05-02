//! Types for the *m.room.redaction* event.

use std::time::SystemTime;

use ruma_events_macros::{Event, EventContent};
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{Deserialize, Serialize};

use crate::{
    MessageEventContent, RedactedMessageEventContent, RedactedStateEventContent, RoomEventContent,
    Unsigned,
};

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
    pub origin_server_ts: SystemTime,

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
    pub origin_server_ts: SystemTime,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: Unsigned,
}

/// A redaction of an event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "m.room.redaction")]
pub struct RedactionEventContent {
    /// The reason for the redaction, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl RoomEventContent for RedactionEventContent {}

impl MessageEventContent for RedactionEventContent {}

impl RedactedMessageEventContent for RedactedRedactionEventContent {}

impl RedactedStateEventContent for RedactedRedactionEventContent {}
