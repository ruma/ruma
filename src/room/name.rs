//! Types for the *m.room.name* event.

use js_int::UInt;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{util::empty_string_as_none, EventType, InvalidInput, TryFromRaw};

/// A human-friendly room name designed to be displayed to the end-user.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename = "m.room.name", tag = "type")]
pub struct NameEvent {
    /// The event's content.
    pub content: NameEventContent,

    /// The unique identifier for the event.
    pub event_id: EventId,

    /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver when this
    /// event was sent.
    pub origin_server_ts: UInt,

    /// The previous content for this state key, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_content: Option<NameEventContent>,

    /// The unique identifier for the room associated with this event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<RoomId>,

    /// The unique identifier for the user who sent this event.
    pub sender: UserId,

    /// A key that determines which piece of room state the event represents.
    pub state_key: String,

    /// Additional key-value pairs not signed by the homeserver.
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub unsigned: Map<String, Value>,
}

/// The payload for `NameEvent`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NameEventContent {
    /// The name of the room. This MUST NOT exceed 255 bytes.
    pub(crate) name: Option<String>,
}

impl TryFromRaw for NameEvent {
    type Raw = raw::NameEvent;
    type Err = InvalidInput;

    fn try_from_raw(raw: Self::Raw) -> Result<Self, Self::Err> {
        let content = TryFromRaw::try_from_raw(raw.content)?;
        let prev_content = raw.prev_content.map(TryFromRaw::try_from_raw).transpose()?;

        Ok(NameEvent {
            content,
            event_id: raw.event_id,
            origin_server_ts: raw.origin_server_ts,
            prev_content,
            room_id: raw.room_id,
            sender: raw.sender,
            state_key: raw.state_key,
            unsigned: raw.unsigned,
        })
    }
}

impl TryFromRaw for NameEventContent {
    type Raw = raw::NameEventContent;

    type Err = InvalidInput;

    fn try_from_raw(raw: raw::NameEventContent) -> Result<Self, Self::Err> {
        match raw.name {
            None => Ok(NameEventContent { name: None }),
            Some(name) => NameEventContent::new(name),
        }
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

pub(crate) mod raw {
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
        #[serde(default)]
        pub unsigned: Map<String, Value>,
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
    use std::{convert::TryFrom, iter::FromIterator};

    use js_int::UInt;
    use ruma_identifiers::{EventId, RoomId, UserId};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value, Map};

    use crate::EventResult;

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
            unsigned: Map::new(),
        };

        let actual = to_json_value(&name_event).unwrap();
        let expected = json!({
            "content": {
                "name": "The room name"
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.name"
        });

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
            unsigned: serde_json::from_str(r#"{"foo": "bar"}"#).unwrap(),
        };

        let actual = to_json_value(&name_event).unwrap();
        let expected = json!({
            "content": {
                "name": "The room name"
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "prev_content": {"name": "The old name"},
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.name",
            "unsigned": {
                "foo": "bar"
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn absent_field_as_none() {
        let json_data = json!({
            "content": {},
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.name"
        });
        assert_eq!(
            from_json_value::<EventResult<NameEvent>>(json_data)
                .unwrap()
                .into_result()
                .unwrap()
                .content
                .name,
            None
        );
    }

    #[test]
    fn name_fails_validation_when_too_long() {
        // "XXXX .." 256 times
        let long_string: String = String::from_iter(std::iter::repeat('X').take(256));
        assert_eq!(long_string.len(), 256);

        let long_content_json_string: String =
            serde_json::json!({ "name": &long_string }).to_string();

        let from_raw: EventResult<NameEventContent> =
            serde_json::from_str(&long_content_json_string).unwrap();

        let result = from_raw.into_result();
        assert!(result.is_err(), "Result should be invalid: {:?}", result);
    }

    #[test]
    fn json_with_empty_name_creates_content_as_none() {
        let long_content_json_string: String = serde_json::json!({ "name": "" }).to_string();

        let from_raw: EventResult<NameEventContent> =
            serde_json::from_str(&long_content_json_string).unwrap();
        assert_eq!(
            from_raw.into_result().unwrap(),
            NameEventContent { name: None }
        );
    }

    #[test]
    fn new_with_empty_name_creates_content_as_none() {
        assert_eq!(
            NameEventContent::new(String::new()).unwrap(),
            NameEventContent { name: None }
        );
    }

    #[test]
    fn null_field_as_none() {
        let json_data = json!({
            "content": {
                "name": null
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.name"
        });
        assert_eq!(
            from_json_value::<EventResult<NameEvent>>(json_data)
                .unwrap()
                .into_result()
                .unwrap()
                .content
                .name,
            None
        );
    }

    #[test]
    fn empty_string_as_none() {
        let json_data = json!({
            "content": {
                "name": ""
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.name"
        });
        assert_eq!(
            from_json_value::<EventResult<NameEvent>>(json_data)
                .unwrap()
                .into_result()
                .unwrap()
                .content
                .name,
            None
        );
    }

    #[test]
    fn nonempty_field_as_some() {
        let name = Some("The room name".to_string());
        let json_data = json!({
            "content": {
                "name": "The room name"
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.name"
        });

        assert_eq!(
            from_json_value::<EventResult<NameEvent>>(json_data)
                .unwrap()
                .into_result()
                .unwrap()
                .content
                .name,
            name
        );
    }
}
