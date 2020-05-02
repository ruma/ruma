//! Types for custom events outside of the Matrix specification.

use std::time::SystemTime;

use crate::{Event, EventType, RoomEvent, StateEvent, UnsignedData};

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

impl Event for CustomEvent {
    /// The type of this event's `content` field.
    type Content = CustomEventContent;

    /// The event's content.
    fn content(&self) -> &Self::Content {
        &self.content
    }

    /// The type of the event.
    fn event_type(&self) -> EventType {
        EventType::Custom(self.event_type.clone())
    }
}

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

impl Event for CustomRoomEvent {
    /// The type of this event's `content` field.
    type Content = CustomRoomEventContent;

    /// The event's content.
    fn content(&self) -> &Self::Content {
        &self.content
    }

    /// The type of the event.
    fn event_type(&self) -> EventType {
        EventType::Custom(self.event_type.clone())
    }
}

impl RoomEvent for CustomRoomEvent {
    /// The unique identifier for the event.
    fn event_id(&self) -> &EventId {
        &self.event_id
    }

    /// Time on originating homeserver when this event was sent.
    fn origin_server_ts(&self) -> SystemTime {
        self.origin_server_ts
    }

    /// The unique identifier for the room associated with this event.
    ///
    /// This can be `None` if the event came from a context where there is
    /// no ambiguity which room it belongs to, like a `/sync` response for example.
    fn room_id(&self) -> Option<&RoomId> {
        self.room_id.as_ref()
    }

    /// The unique identifier for the user who sent this event.
    fn sender(&self) -> &UserId {
        &self.sender
    }

    /// Additional key-value pairs not signed by the homeserver.
    fn unsigned(&self) -> &UnsignedData {
        &self.unsigned
    }
}

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

impl Event for CustomStateEvent {
    /// The type of this event's `content` field.
    type Content = CustomStateEventContent;

    /// The event's content.
    fn content(&self) -> &Self::Content {
        &self.content
    }

    /// The type of the event.
    fn event_type(&self) -> EventType {
        EventType::Custom(self.event_type.clone())
    }
}

impl RoomEvent for CustomStateEvent {
    /// The unique identifier for the event.
    fn event_id(&self) -> &EventId {
        &self.event_id
    }

    /// Time on originating homeserver when this event was sent.
    fn origin_server_ts(&self) -> SystemTime {
        self.origin_server_ts
    }

    /// The unique identifier for the room associated with this event.
    ///
    /// This can be `None` if the event came from a context where there is
    /// no ambiguity which room it belongs to, like a `/sync` response for example.
    fn room_id(&self) -> Option<&RoomId> {
        self.room_id.as_ref()
    }

    /// The unique identifier for the user who sent this event.
    fn sender(&self) -> &UserId {
        &self.sender
    }

    /// Additional key-value pairs not signed by the homeserver.
    fn unsigned(&self) -> &UnsignedData {
        &self.unsigned
    }
}

impl StateEvent for CustomStateEvent {
    /// The previous content for this state key, if any.
    fn prev_content(&self) -> Option<&Self::Content> {
        self.prev_content.as_ref()
    }

    /// A key that determines which piece of room state the event represents.
    fn state_key(&self) -> &str {
        &self.state_key
    }
}

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
