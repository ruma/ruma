//! Types for the *m.room.canonical_alias* event.

use std::{convert::TryFrom, str::FromStr};

use js_int::UInt;
use ruma_identifiers::{EventId, RoomAliasId, RoomId, UserId};
use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::{
    empty_string_as_none, Event, EventResult, EventType, InnerInvalidEvent, InvalidEvent,
    RoomEvent, StateEvent,
};

/// Informs the room as to which alias is the canonical one.
#[derive(Clone, Debug, PartialEq)]
pub struct CanonicalAliasEvent {
    /// The event's content.
    pub content: CanonicalAliasEventContent,

    /// The unique identifier for the event.
    pub event_id: EventId,

    /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver when this
    /// event was sent.
    pub origin_server_ts: UInt,

    /// The previous content for this state key, if any.
    pub prev_content: Option<CanonicalAliasEventContent>,

    /// The unique identifier for the room associated with this event.
    pub room_id: Option<RoomId>,

    /// The unique identifier for the user who sent this event.
    pub sender: UserId,

    /// A key that determines which piece of room state the event represents.
    pub state_key: String,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: Option<Value>,
}

/// The payload for `CanonicalAliasEvent`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CanonicalAliasEventContent {
    /// The canonical alias.
    ///
    /// Rooms with `alias: None` should be treated the same as a room with no canonical alias.
    pub alias: Option<RoomAliasId>,
}

impl<'de> Deserialize<'de> for EventResult<CanonicalAliasEvent> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = serde_json::Value::deserialize(deserializer)?;

        let raw: raw::CanonicalAliasEvent = match serde_json::from_value(json.clone()) {
            Ok(raw) => raw,
            Err(error) => {
                return Ok(EventResult::Err(InvalidEvent(
                    InnerInvalidEvent::Validation {
                        json,
                        message: error.to_string(),
                    },
                )));
            }
        };

        Ok(EventResult::Ok(CanonicalAliasEvent {
            content: CanonicalAliasEventContent {
                alias: raw.content.alias,
            },
            event_id: raw.event_id,
            origin_server_ts: raw.origin_server_ts,
            prev_content: raw
                .prev_content
                .map(|prev| CanonicalAliasEventContent { alias: prev.alias }),
            room_id: raw.room_id,
            sender: raw.sender,
            state_key: raw.state_key,
            unsigned: raw.unsigned,
        }))
    }
}

impl FromStr for CanonicalAliasEvent {
    type Err = InvalidEvent;

    /// Attempt to create `Self` from parsing a string of JSON data.
    fn from_str(json: &str) -> Result<Self, Self::Err> {
        let raw = match serde_json::from_str::<raw::CanonicalAliasEvent>(json) {
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
            content: CanonicalAliasEventContent {
                alias: raw.content.alias,
            },
            event_id: raw.event_id,
            origin_server_ts: raw.origin_server_ts,
            prev_content: raw
                .prev_content
                .map(|prev| CanonicalAliasEventContent { alias: prev.alias }),
            room_id: raw.room_id,
            sender: raw.sender,
            state_key: raw.state_key,
            unsigned: raw.unsigned,
        })
    }
}

impl<'a> TryFrom<&'a str> for CanonicalAliasEvent {
    type Error = InvalidEvent;

    /// Attempt to create `Self` from parsing a string of JSON data.
    fn try_from(json: &'a str) -> Result<Self, Self::Error> {
        FromStr::from_str(json)
    }
}

impl Serialize for CanonicalAliasEvent {
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

        let mut state = serializer.serialize_struct("CanonicalAliasEvent", len)?;

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

impl_state_event!(
    CanonicalAliasEvent,
    CanonicalAliasEventContent,
    EventType::RoomCanonicalAlias
);

impl<'de> Deserialize<'de> for EventResult<CanonicalAliasEventContent> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = serde_json::Value::deserialize(deserializer)?;

        let raw: raw::CanonicalAliasEventContent = match serde_json::from_value(json.clone()) {
            Ok(raw) => raw,
            Err(error) => {
                return Ok(EventResult::Err(InvalidEvent(
                    InnerInvalidEvent::Validation {
                        json,
                        message: error.to_string(),
                    },
                )));
            }
        };

        Ok(EventResult::Ok(CanonicalAliasEventContent {
            alias: raw.alias,
        }))
    }
}

impl FromStr for CanonicalAliasEventContent {
    type Err = InvalidEvent;

    /// Attempt to create `Self` from parsing a string of JSON data.
    fn from_str(json: &str) -> Result<Self, Self::Err> {
        let raw = match serde_json::from_str::<raw::CanonicalAliasEventContent>(json) {
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

        Ok(Self { alias: raw.alias })
    }
}

impl<'a> TryFrom<&'a str> for CanonicalAliasEventContent {
    type Error = InvalidEvent;

    /// Attempt to create `Self` from parsing a string of JSON data.
    fn try_from(json: &'a str) -> Result<Self, Self::Error> {
        FromStr::from_str(json)
    }
}

mod raw {
    use super::*;

    /// Informs the room as to which alias is the canonical one.
    #[derive(Clone, Debug, Deserialize, PartialEq)]
    pub struct CanonicalAliasEvent {
        /// The event's content.
        pub content: CanonicalAliasEventContent,

        /// The unique identifier for the event.
        pub event_id: EventId,

        /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver when this
        /// event was sent.
        pub origin_server_ts: UInt,

        /// The previous content for this state key, if any.
        pub prev_content: Option<CanonicalAliasEventContent>,

        /// The unique identifier for the room associated with this event.
        pub room_id: Option<RoomId>,

        /// The unique identifier for the user who sent this event.
        pub sender: UserId,

        /// A key that determines which piece of room state the event represents.
        pub state_key: String,

        /// Additional key-value pairs not signed by the homeserver.
        pub unsigned: Option<Value>,
    }

    /// The payload of a `CanonicalAliasEvent`.
    #[derive(Clone, Debug, Deserialize, PartialEq)]
    pub struct CanonicalAliasEventContent {
        /// The canonical alias.
        ///
        /// Rooms with `alias: None` should be treated the same as a room with no canonical alias.
        // The spec says "A room with an m.room.canonical_alias event with an absent, null, or empty
        // alias field should be treated the same as a room with no m.room.canonical_alias event."
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub alias: Option<RoomAliasId>,
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use js_int::UInt;
    use ruma_identifiers::{EventId, RoomAliasId, UserId};

    use super::{CanonicalAliasEvent, CanonicalAliasEventContent, EventResult};

    #[test]
    fn serialization_with_optional_fields_as_none() {
        let canonical_alias_event = CanonicalAliasEvent {
            content: CanonicalAliasEventContent {
                alias: Some(RoomAliasId::try_from("#somewhere:localhost").unwrap()),
            },
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UInt::try_from(1).unwrap(),
            prev_content: None,
            room_id: None,
            sender: UserId::try_from("@carl:example.com").unwrap(),
            state_key: "".to_string(),
            unsigned: None,
        };

        let actual = serde_json::to_string(&canonical_alias_event).unwrap();
        let expected = r##"{"content":{"alias":"#somewhere:localhost"},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"sender":"@carl:example.com","state_key":"","type":"m.room.canonical_alias"}"##;

        assert_eq!(actual, expected);
    }

    #[test]
    fn absent_field_as_none() {
        assert_eq!(
            serde_json::from_str::<EventResult<CanonicalAliasEvent>>(
                r#"{"content":{},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"sender":"@carl:example.com","state_key":"","type":"m.room.canonical_alias"}"#
            )
                .unwrap()
                .into_result()
                .unwrap()
                .content
                .alias,
            None
        );
    }

    #[test]
    fn null_field_as_none() {
        assert_eq!(
            serde_json::from_str::<EventResult<CanonicalAliasEvent>>(
                r#"{"content":{"alias":null},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"sender":"@carl:example.com","state_key":"","type":"m.room.canonical_alias"}"#
            )
                .unwrap()
                .into_result()
                .unwrap()
                .content
                .alias,
            None
        );
    }

    #[test]
    fn empty_field_as_none() {
        assert_eq!(
            serde_json::from_str::<EventResult<CanonicalAliasEvent>>(
                r#"{"content":{"alias":""},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"sender":"@carl:example.com","state_key":"","type":"m.room.canonical_alias"}"#
            )
                .unwrap()
                .into_result()
                .unwrap()
                .content
                .alias,
            None
        );
    }

    #[test]
    fn nonempty_field_as_some() {
        let alias = Some(RoomAliasId::try_from("#somewhere:localhost").unwrap());

        assert_eq!(
            serde_json::from_str::<EventResult<CanonicalAliasEvent>>(
                r##"{"content":{"alias":"#somewhere:localhost"},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"sender":"@carl:example.com","state_key":"","type":"m.room.canonical_alias"}"##
            )
                .unwrap()
                .into_result()
                .unwrap()
                .content
                .alias,
            alias
        );
    }
}
