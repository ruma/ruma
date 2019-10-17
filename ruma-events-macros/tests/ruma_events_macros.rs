use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

use serde::{
    de::{DeserializeOwned, Error as SerdeError, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_json::Value;

/// The type of an event.
#[derive(Clone, Debug, PartialEq)]
pub enum EventType {
    /// m.direct
    Direct,

    /// m.room.aliases
    RoomAliases,

    /// m.room.redaction
    RoomRedaction,

    /// Any event that is not part of the specification.
    Custom(String),
}

impl Display for EventType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let event_type_str = match *self {
            EventType::Direct => "m.direct",
            EventType::RoomAliases => "m.room.aliases",
            EventType::RoomRedaction => "m.room.redaction",
            EventType::Custom(ref event_type) => event_type,
        };

        write!(f, "{}", event_type_str)
    }
}

impl<'a> From<&'a str> for EventType {
    fn from(s: &'a str) -> EventType {
        match s {
            "m.direct" => EventType::Direct,
            "m.room.aliases" => EventType::RoomAliases,
            "m.room.redaction" => EventType::RoomRedaction,
            event_type => EventType::Custom(event_type.to_string()),
        }
    }
}

impl Serialize for EventType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for EventType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EventTypeVisitor;

        impl<'de> Visitor<'de> for EventTypeVisitor {
            type Value = EventType;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
                write!(formatter, "a Matrix event type as a string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                Ok(EventType::from(v))
            }
        }

        deserializer.deserialize_str(EventTypeVisitor)
    }
}

/// The result of deserializing an event, which may or may not be valid.
#[derive(Debug)]
pub enum EventResult<T: TryFromRaw> {
    /// `T` deserialized and validated successfully.
    Ok(T),

    /// `T` deserialized but was invalid.
    ///
    /// `InvalidEvent` contains the original input.
    Err(InvalidEvent<T::Raw>),
}

impl<T: TryFromRaw> EventResult<T> {
    /// Convert `EventResult<T>` into the equivalent `std::result::Result<T, InvalidEvent<T::Raw>>`.
    pub fn into_result(self) -> Result<T, InvalidEvent<T::Raw>> {
        match self {
            EventResult::Ok(t) => Ok(t),
            EventResult::Err(invalid_event) => Err(invalid_event),
        }
    }
}

/// Marks types that can be deserialized as EventResult<Self> (and don't need fallible conversion
/// from their raw type)
pub trait FromRaw: Sized {
    /// The raw form of this event that deserialization falls back to if deserializing `Self` fails.
    type Raw: DeserializeOwned;

    fn from_raw(_: Self::Raw) -> Self;
}

pub trait TryFromRaw: Sized {
    /// The raw form of this event that deserialization falls back to if deserializing `Self` fails.
    type Raw: DeserializeOwned;
    type Err: Into<String>;

    fn try_from_raw(_: Self::Raw) -> Result<Self, (Self::Err, Self::Raw)>;
}

impl<T: FromRaw> TryFromRaw for T {
    type Raw = <T as FromRaw>::Raw;
    type Err = Void;

    fn try_from_raw(raw: Self::Raw) -> Result<Self, (Self::Err, Self::Raw)> {
        Ok(Self::from_raw(raw))
    }
}

impl<'de, T> Deserialize<'de> for EventResult<T>
where
    T: TryFromRaw,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = serde_json::Value::deserialize(deserializer)?;

        let raw_data: T::Raw = match serde_json::from_value(json.clone()) {
            Ok(raw) => raw,
            Err(error) => {
                return Ok(EventResult::Err(InvalidEvent(
                    InnerInvalidEvent::Deserialization {
                        json,
                        error: error.to_string(),
                    },
                )));
            }
        };

        match T::try_from_raw(raw_data) {
            Ok(value) => Ok(EventResult::Ok(value)),
            Err((msg, raw_data)) => Ok(EventResult::Err(InvalidEvent(
                InnerInvalidEvent::Validation {
                    message: msg.into(),
                    raw_data,
                },
            ))),
        }
    }
}

pub enum Void {}

impl From<Void> for String {
    fn from(v: Void) -> Self {
        match v {}
    }
}

/// A basic event.
pub trait Event: Debug + Serialize + TryFromRaw {
    /// The type of this event's `content` field.
    type Content: Debug + Serialize;

    /// The event's content.
    fn content(&self) -> &Self::Content;

    /// The type of the event.
    fn event_type(&self) -> EventType;
}

/// An event within the context of a room.
pub trait RoomEvent: Event {
    /// The unique identifier for the event.
    fn event_id(&self) -> &ruma_identifiers::EventId;

    /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver when this event was
    /// sent.
    fn origin_server_ts(&self) -> js_int::UInt;

    /// The unique identifier for the room associated with this event.
    ///
    /// This can be `None` if the event came from a context where there is
    /// no ambiguity which room it belongs to, like a `/sync` response for example.
    fn room_id(&self) -> Option<&ruma_identifiers::RoomId>;

    /// The unique identifier for the user who sent this event.
    fn sender(&self) -> &ruma_identifiers::UserId;

    /// Additional key-value pairs not signed by the homeserver.
    fn unsigned(&self) -> Option<&serde_json::Value>;
}

/// An event that describes persistent state about a room.
pub trait StateEvent: RoomEvent {
    /// The previous content for this state key, if any.
    fn prev_content(&self) -> Option<&Self::Content>;

    /// A key that determines which piece of room state the event represents.
    fn state_key(&self) -> &str;
}

/// An event that is malformed or otherwise invalid.
///
/// When attempting to create an event from a string of JSON data, an error in the input data may
/// cause deserialization to fail, or the JSON structure may not corresponded to ruma-events's
/// strict definition of the event's schema. If deserialization completely fails, this type will
/// provide a message with details about the deserialization error. If deserialization succeeds but
/// the event is otherwise invalid, a similar message will be provided, as well as a
/// `serde_json::Value` containing the raw JSON data as it was deserialized.
#[derive(Debug)]
pub struct InvalidEvent<T>(InnerInvalidEvent<T>);

/// An event that is malformed or otherwise invalid.
#[derive(Debug)]
enum InnerInvalidEvent<T> {
    /// An event that failed to deserialize from JSON.
    Deserialization {
        /// The raw `serde_json::Value` representation of the invalid event.
        json: Value,

        /// The deserialization error returned by serde.
        error: String,
    },

    /// An event that deserialized but failed validation.
    Validation {
        /// The event data that failed validation.
        raw_data: T,

        /// An message describing why the event was invalid.
        message: String,
    },
}

// See note about wrapping macro expansion in a module from `src/lib.rs`
pub mod common_case {
    use std::convert::TryFrom;

    use js_int::UInt;
    use ruma_events_macros::ruma_event;
    use ruma_identifiers::{EventId, RoomAliasId, RoomId, UserId};
    use serde_json::Value;

    use super::EventResult;

    ruma_event! {
        /// Informs the room about what room aliases it has been given.
        AliasesEvent {
            kind: StateEvent,
            event_type: RoomAliases,
            content: {
                /// A list of room aliases.
                pub aliases: Vec<ruma_identifiers::RoomAliasId>,
            }
        }
    }

    #[test]
    fn serialization_with_optional_fields_as_none() {
        let event = AliasesEvent {
            content: AliasesEventContent {
                aliases: Vec::with_capacity(0),
            },
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UInt::try_from(1).unwrap(),
            prev_content: None,
            room_id: None,
            sender: UserId::try_from("@carl:example.com").unwrap(),
            state_key: "example.com".to_string(),
            unsigned: None,
        };

        let actual = serde_json::to_string(&event).unwrap();
        let expected = r#"{"content":{"aliases":[]},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"sender":"@carl:example.com","state_key":"example.com","type":"m.room.aliases"}"#;

        assert_eq!(actual, expected);
    }

    #[test]
    fn serialization_with_some_optional_fields_as_some() {
        let event = AliasesEvent {
            content: AliasesEventContent {
                aliases: vec![RoomAliasId::try_from("#room:example.org").unwrap()],
            },
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UInt::try_from(1).unwrap(),
            prev_content: Some(AliasesEventContent {
                aliases: Vec::with_capacity(0),
            }),
            room_id: Some(RoomId::try_from("!n8f893n9:example.com").unwrap()),
            sender: UserId::try_from("@carl:example.com").unwrap(),
            state_key: "example.com".to_string(),
            unsigned: None,
        };

        let actual = serde_json::to_string(&event).unwrap();
        let expected = r##"{"content":{"aliases":["#room:example.org"]},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"prev_content":{"aliases":[]},"room_id":"!n8f893n9:example.com","sender":"@carl:example.com","state_key":"example.com","type":"m.room.aliases"}"##;

        assert_eq!(actual, expected);
    }

    #[test]
    fn serialization_with_all_optional_fields_as_some() {
        let event = AliasesEvent {
            content: AliasesEventContent {
                aliases: vec![RoomAliasId::try_from("#room:example.org").unwrap()],
            },
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UInt::try_from(1).unwrap(),
            prev_content: Some(AliasesEventContent {
                aliases: Vec::with_capacity(0),
            }),
            room_id: Some(RoomId::try_from("!n8f893n9:example.com").unwrap()),
            sender: UserId::try_from("@carl:example.com").unwrap(),
            state_key: "example.com".to_string(),
            unsigned: Some(serde_json::from_str::<Value>(r#"{"foo":"bar"}"#).unwrap()),
        };

        let actual = serde_json::to_string(&event).unwrap();
        let expected = r##"{"content":{"aliases":["#room:example.org"]},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"prev_content":{"aliases":[]},"room_id":"!n8f893n9:example.com","sender":"@carl:example.com","state_key":"example.com","unsigned":{"foo":"bar"},"type":"m.room.aliases"}"##;

        assert_eq!(actual, expected);
    }

    #[test]
    fn deserialization() {
        let json = r##"{"content":{"aliases":["#room:example.org"]},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"prev_content":{"aliases":[]},"room_id":"!n8f893n9:example.com","sender":"@carl:example.com","state_key":"example.com","unsigned":{"foo":"bar"},"type":"m.room.aliases"}"##;

        let event_result: EventResult<AliasesEvent> = serde_json::from_str(json).unwrap();
        let actual: AliasesEvent = event_result.into_result().unwrap();

        let expected = AliasesEvent {
            content: AliasesEventContent {
                aliases: vec![RoomAliasId::try_from("#room:example.org").unwrap()],
            },
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UInt::try_from(1).unwrap(),
            prev_content: Some(AliasesEventContent {
                aliases: Vec::with_capacity(0),
            }),
            room_id: Some(RoomId::try_from("!n8f893n9:example.com").unwrap()),
            sender: UserId::try_from("@carl:example.com").unwrap(),
            state_key: "example.com".to_string(),
            unsigned: Some(serde_json::from_str::<Value>(r#"{"foo":"bar"}"#).unwrap()),
        };

        assert_eq!(actual, expected);
    }
}

pub mod custom_event_type {
    use ruma_events_macros::ruma_event;
    use serde_json::Value;

    ruma_event! {
        /// A custom event.
        CustomEvent {
            kind: Event,
            event_type: Custom,
            content_type_alias: {
                /// The payload for `CustomEvent`.
                Value
            },
        }
    }
}

pub mod extra_fields {
    use ruma_events_macros::ruma_event;

    ruma_event! {
        /// A redaction of an event.
        RedactionEvent {
            kind: RoomEvent,
            event_type: RoomRedaction,
            fields: {
                /// The ID of the event that was redacted.
                pub redacts: ruma_identifiers::EventId
            },
            content: {
                /// The reason for the redaction, if any.
                pub reason: Option<String>,
            },
        }
    }
}

pub mod type_alias {
    use ruma_events_macros::ruma_event;

    ruma_event! {
        /// Informs the client about the rooms that are considered direct by a user.
        DirectEvent {
            kind: Event,
            event_type: Direct,
            content_type_alias: {
                /// The payload of a `DirectEvent`.
                ///
                /// A mapping of `UserId`'s to a collection of `RoomId`'s which are considered
                /// *direct* for that particular user.
                std::collections::HashMap<ruma_identifiers::UserId, Vec<ruma_identifiers::RoomId>>
            }
        }
    }
}
