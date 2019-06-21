//! Types for the *m.room.name* event.

use std::{convert::TryFrom, str::FromStr};

use js_int::UInt;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use serde_json::Value;

use crate::{
    empty_string_as_none, Event, EventType, InvalidEvent, InvalidInput, RoomEvent, StateEvent,
};

/// A human-friendly room name designed to be displayed to the end-user.
#[derive(Clone, Debug, PartialEq)]
pub struct NameEvent {
    /// The event's content.
    pub content: NameEventContent,

    /// The unique identifier for the event.
    pub event_id: EventId,

    /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver when this
    /// event was sent.
    pub origin_server_ts: UInt,

    /// The previous content for this state key, if any.
    pub prev_content: Option<NameEventContent>,

    /// The unique identifier for the room associated with this event.
    pub room_id: Option<RoomId>,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: Option<Value>,

    /// The unique identifier for the user who sent this event.
    pub sender: UserId,

    /// A key that determines which piece of room state the event represents.
    pub state_key: String,
}

/// The payload of a `NameEvent`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NameEventContent {
    /// The name of the room. This MUST NOT exceed 255 bytes.
    // The spec says “A room with an m.room.name event with an absent, null, or empty name field
    // should be treated the same as a room with no m.room.name event.”.
    // Serde maps null fields to None by default, serde(default) maps an absent field to None,
    // and empty_string_as_none completes the handling.
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub(crate) name: Option<String>,
}

impl FromStr for NameEvent {
    type Err = crate::InvalidEvent;

    /// Attempt to create `Self` from parsing a string of JSON data.
    fn from_str(json: &str) -> Result<Self, Self::Err> {
        let raw = serde_json::from_str::<raw::NameEvent>(json)?;

        Ok(Self {
            content: NameEventContent {
                name: raw.content.name,
            },
            event_id: raw.event_id,
            origin_server_ts: raw.origin_server_ts,
            prev_content: raw
                .prev_content
                .map(|prev| NameEventContent { name: prev.name }),
            room_id: raw.room_id,
            unsigned: raw.unsigned,
            sender: raw.sender,
            state_key: raw.state_key,
        })
    }
}

impl<'a> TryFrom<&'a str> for NameEvent {
    type Error = InvalidEvent;

    /// Attempt to create `Self` from parsing a string of JSON data.
    fn try_from(json: &'a str) -> Result<Self, Self::Error> {
        FromStr::from_str(json)
    }
}

impl Serialize for NameEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("NameEvent", 2)?;

        state.serialize_field("content", &self.content)?;
        state.serialize_field("type", &self.event_type())?;

        state.end()
    }
}

impl Event for NameEvent {
    /// The type of this event's `content` field.
    type Content = NameEventContent;

    /// The event's content.
    fn content(&self) -> &Self::Content {
        &self.content
    }

    /// The type of the event.
    fn event_type(&self) -> EventType {
        EventType::RoomName
    }
}

impl RoomEvent for NameEvent {
    /// The unique identifier for the event.
    fn event_id(&self) -> &EventId {
        &self.event_id
    }

    /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver when this event was
    /// sent.
    fn origin_server_ts(&self) -> UInt {
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
    fn unsigned(&self) -> Option<&Value> {
        self.unsigned.as_ref()
    }
}

impl StateEvent for NameEvent {
    /// The previous content for this state key, if any.
    fn prev_content(&self) -> Option<&Self::Content> {
        self.prev_content.as_ref()
    }

    /// A key that determines which piece of room state the event represents.
    fn state_key(&self) -> &str {
        &self.state_key
    }
}

impl NameEventContent {
    /// Create a new `NameEventContent` with the given name.
    ///
    /// # Errors
    ///
    /// `InvalidInput` will be returned if the name is more than 255 bytes.
    pub fn new(name: String) -> Result<Self, InvalidInput> {
        match name.len() {
            0 => Ok(Self { name: None }),
            1..=255 => Ok(Self { name: Some(name) }),
            _ => Err(InvalidInput(
                "a room name cannot be more than 255 bytes".to_string(),
            )),
        }
    }

    /// The name of the room, if any.
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(String::as_ref)
    }
}

mod raw {
    use super::*;

    /// A human-friendly room name designed to be displayed to the end-user.
    #[derive(Clone, Debug, Deserialize, PartialEq)]
    pub struct NameEvent {
        /// The event's content.
        pub content: NameEventContent,

        /// The unique identifier for the event.
        pub event_id: EventId,

        /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver when this
        /// event was sent.
        pub origin_server_ts: UInt,

        /// The previous content for this state key, if any.
        pub prev_content: Option<NameEventContent>,

        /// The unique identifier for the room associated with this event.
        pub room_id: Option<RoomId>,

        /// Additional key-value pairs not signed by the homeserver.
        pub unsigned: Option<Value>,

        /// The unique identifier for the user who sent this event.
        pub sender: UserId,

        /// A key that determines which piece of room state the event represents.
        pub state_key: String,
    }

    /// The payload of a `NameEvent`.
    #[derive(Clone, Debug, Deserialize, PartialEq)]
    pub struct NameEventContent {
        /// The name of the room. This MUST NOT exceed 255 bytes.
        // The spec says “A room with an m.room.name event with an absent, null, or empty name field
        // should be treated the same as a room with no m.room.name event.”.
        // Serde maps null fields to None by default, serde(default) maps an absent field to None,
        // and empty_string_as_none completes the handling.
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub(crate) name: Option<String>,
    }
}

#[cfg(test)]
mod tests {
    use super::NameEvent;

    #[test]
    fn absent_field_as_none() {
        assert_eq!(
            r#"{"content":{},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"sender":"@carl:matrix.org","state_key":"","type":"m.room.name"}"#
                .parse::<NameEvent>()
                .unwrap()
                .content
                .name,
            None
        );
    }

    #[test]
    fn null_field_as_none() {
        assert_eq!(
            r#"{"content":{"name":null},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"sender":"@carl:matrix.org","state_key":"","type":"m.room.name"}"#
                .parse::<NameEvent>()
                .unwrap()
                .content
                .name,
            None
        );
    }

    #[test]
    fn empty_string_as_none() {
        assert_eq!(
            r#"{"content":{"name":""},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"sender":"@carl:matrix.org","state_key":"","type":"m.room.name"}"#
                .parse::<NameEvent>()
                .unwrap()
                .content
                .name,
            None
        );
    }

    #[test]
    fn nonempty_field_as_some() {
        let name = Some("The room name".to_string());

        assert_eq!(
            r#"{"content":{"name":"The room name"},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"sender":"@carl:matrix.org","state_key":"","type":"m.room.name"}"#
                .parse::<NameEvent>()
                .unwrap()
                .content
                .name,
            name
        );
    }
}
