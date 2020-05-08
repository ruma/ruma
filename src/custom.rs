//! Types for custom events outside of the Matrix specification.

use std::time::SystemTime;

use crate::{EventType, UnsignedData};

use ruma_events_macros::FromRaw;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::Serialize;
use serde_json::Value as JsonValue;

/// A custom event not covered by the Matrix specification.
#[derive(Clone, Debug, FromRaw, Serialize)]
pub struct CustomEvent {
    /// The event's content.
    pub content: CustomEventContent,
    /// The custom type of the event.
    #[serde(rename = "type")]
    pub event_type: String,
}

/// The payload for `CustomEvent`.
pub type CustomEventContent = JsonValue;

/// A custom room event not covered by the Matrix specification.
#[derive(Clone, Debug, FromRaw, Serialize)]
pub struct CustomRoomEvent {
    /// The event's content.
    pub content: CustomRoomEventContent,
    /// The unique identifier for the event.
    pub event_id: EventId,
    /// The custom type of the event.
    #[serde(rename = "type")]
    pub event_type: String,
    /// Time on originating homeserver when this event was sent.
    #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
    pub origin_server_ts: SystemTime,
    /// The unique identifier for the room associated with this event.
    pub room_id: Option<RoomId>,
    /// The unique identifier for the user who sent this event.
    pub sender: UserId,
    /// Additional key-value pairs not signed by the homeserver.
    #[serde(skip_serializing_if = "UnsignedData::is_empty")]
    pub unsigned: UnsignedData,
}

/// The payload for `CustomRoomEvent`.
pub type CustomRoomEventContent = JsonValue;

/// A custom state event not covered by the Matrix specification.
#[derive(Clone, Debug, FromRaw, Serialize)]
pub struct CustomStateEvent {
    /// The event's content.
    pub content: CustomStateEventContent,
    /// The unique identifier for the event.
    pub event_id: EventId,
    /// The custom type of the event.
    #[serde(rename = "type")]
    pub event_type: String,
    /// Time on originating homeserver when this event was sent.
    #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
    pub origin_server_ts: SystemTime,
    /// The previous content for this state key, if any.
    pub prev_content: Option<CustomStateEventContent>,
    /// The unique identifier for the room associated with this event.
    pub room_id: Option<RoomId>,
    /// The unique identifier for the user who sent this event.
    pub sender: UserId,
    /// A key that determines which piece of room state the event represents.
    pub state_key: String,
    /// Additional key-value pairs not signed by the homeserver.
    #[serde(skip_serializing_if = "UnsignedData::is_empty")]
    pub unsigned: UnsignedData,
}

/// The payload for `CustomStateEvent`.
pub type CustomStateEventContent = JsonValue;

pub(crate) mod raw {
    use std::time::SystemTime;

    use ruma_identifiers::{EventId, RoomId, UserId};
    use serde::Deserialize;

    use super::{
        CustomEventContent, CustomRoomEventContent, CustomStateEventContent, UnsignedData,
    };

    /// A custom event not covered by the Matrix specification.
    #[derive(Clone, Debug, Deserialize)]
    pub struct CustomEvent {
        /// The event's content.
        pub content: CustomEventContent,
        /// The custom type of the event.
        #[serde(rename = "type")]
        pub event_type: String,
    }

    /// A custom room event not covered by the Matrix specification.
    #[derive(Clone, Debug, Deserialize)]
    pub struct CustomRoomEvent {
        /// The event's content.
        pub content: CustomRoomEventContent,
        /// The unique identifier for the event.
        pub event_id: EventId,
        /// The custom type of the event.
        #[serde(rename = "type")]
        pub event_type: String,
        /// Time on originating homeserver when this event was sent.
        #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
        pub origin_server_ts: SystemTime,
        /// The unique identifier for the room associated with this event.
        pub room_id: Option<RoomId>,
        /// The unique identifier for the user who sent this event.
        pub sender: UserId,
        /// Additional key-value pairs not signed by the homeserver.
        #[serde(default)]
        pub unsigned: UnsignedData,
    }

    /// A custom state event not covered by the Matrix specification.
    #[derive(Clone, Debug, Deserialize)]
    pub struct CustomStateEvent {
        /// The event's content.
        pub content: CustomStateEventContent,
        /// The unique identifier for the event.
        pub event_id: EventId,
        /// The custom type of the event.
        #[serde(rename = "type")]
        pub event_type: String,
        /// Time on originating homeserver when this event was sent.
        #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
        pub origin_server_ts: SystemTime,
        /// The previous content for this state key, if any.
        pub prev_content: Option<CustomStateEventContent>,
        /// The unique identifier for the room associated with this event.
        pub room_id: Option<RoomId>,
        /// The unique identifier for the user who sent this event.
        pub sender: UserId,
        /// A key that determines which piece of room state the event represents.
        pub state_key: String,
        /// Additional key-value pairs not signed by the homeserver.
        #[serde(default)]
        pub unsigned: UnsignedData,
    }
}
