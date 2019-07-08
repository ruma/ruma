//! Types for the *m.room.name* event.

use std::{convert::TryFrom, str::FromStr};

use js_int::UInt;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use serde_json::Value;

use crate::{
    empty_string_as_none, Event, EventType, InnerInvalidEvent, InvalidEvent, InvalidInput,
    RoomEvent, StateEvent,
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

    /// The unique identifier for the user who sent this event.
    pub sender: UserId,

    /// A key that determines which piece of room state the event represents.
    pub state_key: String,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: Option<Value>,
}

/// The payload for `NameEvent`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NameEventContent {
    /// The name of the room. This MUST NOT exceed 255 bytes.
    pub(crate) name: Option<String>,
}

impl FromStr for NameEvent {
    type Err = crate::InvalidEvent;

    /// Attempt to create `Self` from parsing a string of JSON data.
    fn from_str(json: &str) -> Result<Self, Self::Err> {
        let raw = match serde_json::from_str::<raw::NameEvent>(json) {
            Ok(raw) => raw,
            Err(error) => match serde_json::from_str::<serde_json::Value>(json) {
                Ok(value) => {
                    return Err(InvalidEvent(InnerInvalidEvent::Validation {
                        json: value,
                        message: error.to_string(),
                    }));
                }
                Err(error) => {
                    return Err(InvalidEvent(InnerInvalidEvent::Deserialization { error }));
                }
            },
        };

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
            sender: raw.sender,
            state_key: raw.state_key,
            unsigned: raw.unsigned,
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
        let mut len = 6;

        if self.prev_content.is_some() {
            len += 1;
        }

        if self.room_id.is_some() {
            len += 1;
        }

        if self.unsigned.is_some() {
            len += 1;
        }

        let mut state = serializer.serialize_struct("NameEvent", len)?;

        state.serialize_field("content", &self.content)?;
        state.serialize_field("event_id", &self.event_id)?;
        state.serialize_field("origin_server_ts", &self.origin_server_ts)?;

        if self.prev_content.is_some() {
            state.serialize_field("prev_content", &self.prev_content)?;
        }

        if self.room_id.is_some() {
            state.serialize_field("room_id", &self.room_id)?;
        }

        state.serialize_field("sender", &self.sender)?;
        state.serialize_field("state_key", &self.state_key)?;
        state.serialize_field("type", &self.event_type())?;

        if self.unsigned.is_some() {
            state.serialize_field("unsigned", &self.unsigned)?;
        }

        state.end()
    }
}

impl_state_event!(NameEvent, NameEventContent, EventType::RoomName);

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

        /// The unique identifier for the user who sent this event.
        pub sender: UserId,

        /// A key that determines which piece of room state the event represents.
        pub state_key: String,

        /// Additional key-value pairs not signed by the homeserver.
        pub unsigned: Option<Value>,
    }

    /// The payload of a `NameEvent`.
    #[derive(Clone, Debug, Deserialize, PartialEq)]
    pub struct NameEventContent {
        /// The name of the room. This MUST NOT exceed 255 bytes.
        // The spec says "A room with an m.room.name event with an absent, null, or empty name field
        // should be treated the same as a room with no m.room.name event."
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub(crate) name: Option<String>,
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use js_int::UInt;
    use ruma_identifiers::{EventId, RoomId, UserId};
    use serde_json::Value;

    use super::{NameEvent, NameEventContent};

    #[test]
    fn serialization_with_optional_fields_as_none() {
        let name_event = NameEvent {
            content: NameEventContent {
                name: Some("The room name".to_string()),
            },
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UInt::try_from(1).unwrap(),
            prev_content: None,
            room_id: None,
            sender: UserId::try_from("@carl:example.com").unwrap(),
            state_key: "".to_string(),
            unsigned: None,
        };

        let actual = serde_json::to_string(&name_event).unwrap();
        let expected = r#"{"content":{"name":"The room name"},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"sender":"@carl:example.com","state_key":"","type":"m.room.name"}"#;

        assert_eq!(actual, expected);
    }

    #[test]
    fn serialization_with_all_fields() {
        let name_event = NameEvent {
            content: NameEventContent {
                name: Some("The room name".to_string()),
            },
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UInt::try_from(1).unwrap(),
            prev_content: Some(NameEventContent {
                name: Some("The old name".to_string()),
            }),
            room_id: Some(RoomId::try_from("!n8f893n9:example.com").unwrap()),
            sender: UserId::try_from("@carl:example.com").unwrap(),
            state_key: "".to_string(),
            unsigned: Some(serde_json::from_str::<Value>(r#"{"foo":"bar"}"#).unwrap()),
        };

        let actual = serde_json::to_string(&name_event).unwrap();
        let expected = r#"{"content":{"name":"The room name"},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"prev_content":{"name":"The old name"},"room_id":"!n8f893n9:example.com","sender":"@carl:example.com","state_key":"","type":"m.room.name","unsigned":{"foo":"bar"}}"#;

        assert_eq!(actual, expected);
    }

    #[test]
    fn absent_field_as_none() {
        assert_eq!(
            r#"{"content":{},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"sender":"@carl:example.com","state_key":"","type":"m.room.name"}"#
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
            r#"{"content":{"name":null},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"sender":"@carl:example.com","state_key":"","type":"m.room.name"}"#
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
            r#"{"content":{"name":""},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"sender":"@carl:example.com","state_key":"","type":"m.room.name"}"#
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
            r#"{"content":{"name":"The room name"},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"sender":"@carl:example.com","state_key":"","type":"m.room.name"}"#
                .parse::<NameEvent>()
                .unwrap()
                .content
                .name,
            name
        );
    }
}
