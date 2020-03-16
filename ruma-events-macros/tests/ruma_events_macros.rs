use std::{
    borrow::Cow,
    collections::HashMap,
    convert::{Infallible, TryFrom},
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};

use js_int::UInt;
use ruma_events_macros::ruma_event;
use ruma_identifiers::{EventId, RoomAliasId, RoomId, UserId};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};

/// The type of an event.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
// Cow<str> because deserialization sometimes needs to copy to unescape things
#[serde(from = "Cow<'_, str>", into = "String")]
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

impl<'a> From<Cow<'a, str>> for EventType {
    fn from(s: Cow<'a, str>) -> EventType {
        match &s as &str {
            "m.direct" => EventType::Direct,
            "m.room.aliases" => EventType::RoomAliases,
            "m.room.redaction" => EventType::RoomRedaction,
            _ => EventType::Custom(s.into_owned()),
        }
    }
}

impl From<&str> for EventType {
    fn from(s: &str) -> EventType {
        EventType::from(Cow::Borrowed(s))
    }
}

impl From<EventType> for String {
    fn from(event_type: EventType) -> String {
        event_type.to_string()
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
    Err(InvalidEvent),
}

impl<T: TryFromRaw> EventResult<T> {
    /// Convert `EventResult<T>` into the equivalent `std::result::Result<T, InvalidEvent>`.
    pub fn into_result(self) -> Result<T, InvalidEvent> {
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
    type Err: Display;

    fn try_from_raw(_: Self::Raw) -> Result<Self, Self::Err>;
}

impl FromRaw for serde_json::Value {
    type Raw = Self;

    fn from_raw(raw: Self) -> Self {
        raw
    }
}

impl<K, V, S> FromRaw for HashMap<K, V, S>
where
    Self: DeserializeOwned,
{
    type Raw = Self;

    fn from_raw(raw: Self) -> Self {
        raw
    }
}

impl<T: FromRaw> TryFromRaw for T {
    type Raw = <T as FromRaw>::Raw;
    type Err = Infallible;

    fn try_from_raw(raw: Self::Raw) -> Result<Self, Self::Err> {
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
                return Ok(EventResult::Err(InvalidEvent {
                    json,
                    message: error.to_string(),
                    kind: InvalidEventKind::Deserialization,
                }));
            }
        };

        match T::try_from_raw(raw_data) {
            Ok(value) => Ok(EventResult::Ok(value)),
            Err(err) => Ok(EventResult::Err(InvalidEvent {
                message: err.to_string(),
                json,
                kind: InvalidEventKind::Validation,
            })),
        }
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
/// When attempting to deserialize an `EventResult`, an error in the input data may cause
/// deserialization to fail, or the JSON structure may be correct, but additional constraints
/// defined in the matrix specification are not upheld. This type provides an error message and a
/// `serde_json::Value` representation of the invalid event, as well as a flag for which type of
/// error was encountered.
#[derive(Clone, Debug)]
pub struct InvalidEvent {
    message: String,
    json: Value,
    kind: InvalidEventKind,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum InvalidEventKind {
    Deserialization,
    Validation,
}

impl InvalidEvent {
    /// A message describing why the event is invalid.
    pub fn message(&self) -> String {
        self.message.clone()
    }

    /// The `serde_json::Value` representation of the invalid event.
    pub fn json(&self) -> &Value {
        &self.json
    }

    /// Returns whether this is a deserialization error.
    pub fn is_deserialization(&self) -> bool {
        self.kind == InvalidEventKind::Deserialization
    }

    /// Returns whether this is a validation error.
    pub fn is_validation(&self) -> bool {
        self.kind == InvalidEventKind::Validation
    }
}

impl Display for InvalidEvent {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

pub fn serde_json_eq_try_from_raw<T>(de: T, se: serde_json::Value)
where
    T: Clone + Debug + PartialEq + Serialize + TryFromRaw,
{
    assert_eq!(se, serde_json::to_value(de.clone()).unwrap());
    assert_eq!(
        de,
        serde_json::from_value::<EventResult<_>>(se)
            .unwrap()
            .into_result()
            .unwrap()
    );
}

// See note about wrapping macro expansion in a module from `src/lib.rs`
mod common_case {
    use super::*;

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
    fn optional_fields_as_none() {
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
        let json = json!({
            "content": {
                "aliases": []
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "sender": "@carl:example.com",
            "state_key": "example.com",
            "type": "m.room.aliases"
        });
        serde_json_eq_try_from_raw(event, json);
    }

    #[test]
    fn some_optional_fields_as_some() {
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
        let json = json!({
            "content": {
                "aliases": ["#room:example.org"]
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "prev_content": {
                "aliases": []
            },
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "example.com",
            "type": "m.room.aliases"
        });
        serde_json_eq_try_from_raw(event, json);
    }

    #[test]
    fn all_optional_fields_as_some() {
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
        let json = json!({
            "content": {
                "aliases": ["#room:example.org"]
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "prev_content": {
                "aliases": []
            },
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "example.com",
            "unsigned": {
                "foo": "bar"
            },
            "type": "m.room.aliases"
        });
        serde_json_eq_try_from_raw(event, json);
    }
}

mod custom_event_type {
    use super::*;

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

    #[test]
    fn value_is_not_null() {
        // Hint: serde_json::Value with default feature is sort
        // alphabetically rather than preserve the sequence of json kv
        // pairs. Check:
        // + https://github.com/serde-rs/json/pull/80
        // + https://github.com/serde-rs/json/blob/17d9a5ea9b8e11f01b0fcf13933c4a12d3f8db45/tests/map.rs.
        let event = CustomEvent {
            content: { serde_json::from_str::<Value>(r#"{"alice":["foo", "bar"]}"#).unwrap() },
            event_type: "foo.bar".to_owned(),
        };
        let json = json!({
            "content": {
                "alice": ["foo", "bar"]
            },
            "type": "foo.bar"
        });
        serde_json_eq_try_from_raw(event, json);
    }

    #[test]
    fn value_is_null() {
        let event = CustomEvent {
            content: { Value::Null },
            event_type: "foo.bar".to_owned(),
        };
        let json = json!({
            "content": null,
            "type": "foo.bar"
        });
        serde_json_eq_try_from_raw(event, json);
    }
}

mod extra_fields {
    use super::*;

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

    #[test]
    fn field_serialization_deserialization() {
        let event = RedactionEvent {
            content: RedactionEventContent { reason: None },
            redacts: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UInt::try_from(1).unwrap(),
            room_id: Some(RoomId::try_from("!n8f893n9:example.com").unwrap()),
            sender: UserId::try_from("@carl:example.com").unwrap(),
            unsigned: Some(serde_json::from_str::<Value>(r#"{"foo":"bar"}"#).unwrap()),
        };
        let json = json!({
            "content": {
                "reason": null
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "redacts": "$h29iv0s8:example.com",
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "unsigned": {
                "foo": "bar"
            },
            "type": "m.room.redaction"
        });
        serde_json_eq_try_from_raw(event, json);
    }
}

mod type_alias {
    use super::*;

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
                HashMap<ruma_identifiers::UserId, Vec<ruma_identifiers::RoomId>>
            }
        }
    }

    #[test]
    fn alias_is_not_empty() {
        let content = vec![(
            UserId::try_from("@bob:example.com").unwrap(),
            vec![RoomId::try_from("!n8f893n9:example.com").unwrap()],
        )]
        .into_iter()
        .collect();

        let event = DirectEvent { content };
        let json = json!({
            "content": {
                "@bob:example.com": ["!n8f893n9:example.com"]
            },
            "type": "m.direct"
        });
        serde_json_eq_try_from_raw(event, json);
    }

    #[test]
    fn alias_empty() {
        let content = Default::default();
        let event = DirectEvent { content };
        let json = json!({
            "content": {},
            "type": "m.direct"
        });
        serde_json_eq_try_from_raw(event, json);
    }
}
