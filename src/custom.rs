//! Types for custom events outside of the Matrix specification.

use std::time::SystemTime;

use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::{EventType, UnsignedData};

// TODO: (De)serialization

/// A custom event's type and `content` JSON object.
#[derive(Clone, Debug, Serialize)]
pub struct CustomEventContent {
    /// The event type string.
    #[serde(skip)]
    pub event_type: String,

    /// The actual `content` JSON object.
    pub json: JsonValue,
}

/// A custom event not covered by the Matrix specification.
#[derive(Clone, Debug)]
pub struct CustomBasicEvent {
    /// The event's content.
    pub content: CustomEventContent,
}

/// A custom message event not covered by the Matrix specification.
#[derive(Clone, Debug)]
pub struct CustomMessageEvent {
    /// The event's content.
    pub content: CustomEventContent,

    /// Time on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// The unique identifier for the room associated with this event.
    pub room_id: Option<RoomId>,

    /// The unique identifier for the user who sent this event.
    pub sender: UserId,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: UnsignedData,
}

/// A custom state event not covered by the Matrix specification.
#[derive(Clone, Debug)]
pub struct CustomStateEvent {
    /// The event's content.
    pub content: CustomEventContent,

    /// The unique identifier for the event.
    pub event_id: EventId,

    /// Time on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// The previous content for this state key, if any.
    pub prev_content: Option<CustomEventContent>,

    /// The unique identifier for the room associated with this event.
    pub room_id: Option<RoomId>,

    /// The unique identifier for the user who sent this event.
    pub sender: UserId,

    /// A key that determines which piece of room state the event represents.
    pub state_key: String,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: UnsignedData,
}
