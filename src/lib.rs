//! Crate `ruma_events` contains serializable types for the events in the [Matrix](https://matrix.org)
//! specification that can be shared by client and server code.
//!
//! All data exchanged over Matrix is expressed as an event.
//! Different event types represent different actions, such as joining a room or sending a message.
//! Events are stored and transmitted as simple JSON structures.
//! While anyone can create a new event type for their own purposes, the Matrix specification
//! defines a number of event types which are considered core to the protocol, and Matrix clients
//! and servers must understand their semantics.
//! ruma-events contains Rust types for each of the event types defined by the specification and
//! facilities for extending the event system for custom event types.
//!
//! # Event types
//!
//! ruma-events includes a Rust enum called `EventType`, which provides a simple enumeration of
//! all the event types defined by the Matrix specification. Matrix event types are serialized to
//! JSON strings in [reverse domain name
//! notation](https://en.wikipedia.org/wiki/Reverse_domain_name_notation), although the core event
//! types all use the special "m" TLD, e.g. *m.room.message*.
//! `EventType` also includes a variant called `Custom`, which is a catch-all that stores a string
//! containing the name of any event type that isn't part of the specification.
//! `EventType` is used throughout ruma-events to identify and differentiate between events of
//! different types.
//!
//! # Event kinds
//!
//! Matrix defines three "kinds" of events:
//!
//! 1.  **Events**, which are arbitrary JSON structures that have two required keys:
//!     *   `type`, which specifies the event's type
//!     *   `content`, which is a JSON object containing the "payload" of the event
//! 2.  **Room events**, which are a superset of events and represent actions that occurred within
//!     the context of a Matrix room.
//!     They have at least the following additional keys:
//!     *   `event_id`, which is a unique identifier for the event
//!     *   `room_id`, which is a unique identifier for the room in which the event occurred
//!     *   `sender`, which is the unique identifier of the Matrix user who created the event
//!     *   Optionally, `unsigned`, which is a JSON object containing arbitrary additional metadata
//!     that is not digitally signed by Matrix homeservers.
//! 3.  **State events**, which are a superset of room events and represent persistent state
//!     specific to a room, such as the room's member list or topic.
//!     Within a single room, state events of the same type and with the same "state key" will
//!     effectively "replace" the previous one, updating the room's state.
//!     They have at least the following additional keys:
//!     *   `state_key`, a string which serves as a sort of "sub-type."
//!         The state key allows a room to persist multiple state events of the same type.
//!         You can think of a room's state events as being a `HashMap` where the keys are the tuple
//!         `(event_type, state_key)`.
//!     *   Optionally, `prev_content`, a JSON object containing the `content` object from the
//!     previous event of the given `(event_type, state_key)` tuple in the given room.
//!
//! ruma-events represents these three event kinds as traits, allowing any Rust type to serve as a
//! Matrix event so long as it upholds the contract expected of its kind.
//!
//! # Core event types
//!
//! ruma-events includes Rust types for every one of the event types in the Matrix specification.
//! To better organize the crate, these types live in separate modules with a hierarchy that
//! matches the reverse domain name notation of the event type.
//! For example, the *m.room.message* event lives at `ruma_events::room::message::MessageEvent`.
//! Each type's module also contains a Rust type for that event type's `content` field, and any
//! other supporting types required by the event's other fields.
//!
//! # Custom event types
//!
//! Although any Rust type that implements `Event`, `RoomEvent`, or `StateEvent` can serve as a
//! Matrix event type, ruma-events also includes a few convenience types for representing events
//! that are not covered by the spec and not otherwise known by the application.
//! `CustomEvent`, `CustomRoomEvent`, and `CustomStateEvent` are simple implementations of their
//! respective event traits whose `content` field is simply a `serde_json::Value` value, which
//! represents arbitrary JSON.
//!
//! # Serialization and deserialization
//!
//! All concrete event types in ruma-events can be serialized via the `Serialize` trait from
//! [serde](https://serde.rs/) and can be deserialized from as `EventResult<EventType>`. In order to
//! handle incoming data that may not conform to `ruma-events`' strict definitions of event
//! structures, deserialization will return `EventResult::Err` on error. This error covers both
//! structurally invalid JSON data as well as structurally valid JSON that doesn't fulfill
//! additional constraints the matrix specification defines for some event types. The error exposes
//! the deserialized `serde_json::Value` so that developers can still work with the received
//! event data. This makes it possible to deserialize a collection of events without the entire
//! collection failing to deserialize due to a single invalid event. The "content" type for each
//! event also implements `Serialize` and either `TryFromRaw` (enabling usage as
//! `EventResult<ContentType>` for dedicated content types) or `Deserialize` (when the content is a
//! type alias), allowing content to be converted to and from JSON indepedently of the surrounding
//! event structure, if needed.
//!
//! # Collections
//!
//! With the trait-based approach to events, it's easy to write generic collection types like
//! `Vec<Box<R: RoomEvent>>`.
//! However, there are APIs in the Matrix specification that involve heterogeneous collections of
//! events, i.e. a list of events of different event types.
//! Because Rust does not have a facility for arrays, vectors, or slices containing multiple
//! concrete types, ruma-events provides special collection types for this purpose.
//! The collection types are enums which effectively "wrap" each possible event type of a
//! particular event "kind."
//!
//! Because of the hierarchical nature of event kinds in Matrix, these collection types are divied
//! into two modules, `ruma_events::collections::all` and `ruma_events::collections::only`.
//! The "all" versions include every event type that implements the relevant event trait as well as
//! more specific event traits.
//! The "only" versions include only the event types that implement "at most" the relevant event
//! trait.
//!
//! For example, the `ruma_events::collections::all::Event` enum includes *m.room.message*, because
//! that event type is both an event and a room event.
//! However, the `ruma_events::collections::only::Event` enum does *not* include *m.room.message*,
//! because *m.room.message* implements a *more specific* event trait than `Event`.

#![warn(rust_2018_idioms)]
#![deny(missing_debug_implementations, missing_docs)]
// Since we support Rust 1.36.0, we can't apply this suggestion yet
#![allow(clippy::use_self)]

use std::{
    convert::Infallible,
    error::Error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};

use js_int::UInt;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{
    de::{DeserializeOwned, MapAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_json::Value;

pub use self::{custom::CustomEvent, custom_room::CustomRoomEvent, custom_state::CustomStateEvent};

#[macro_use]
mod macros;
mod algorithm;
mod event_type;
mod util;

pub mod call;
/// Enums for heterogeneous collections of events.
pub mod collections {
    pub mod all;
    pub mod only;

    mod raw {
        pub mod all;
        pub mod only;
    }
}
pub mod direct;
pub mod dummy;
pub mod forwarded_room_key;
pub mod fully_read;
pub mod ignored_user_list;
pub mod key;
pub mod presence;
pub mod push_rules;
pub mod receipt;
pub mod room;
pub mod room_key;
pub mod room_key_request;
pub mod sticker;
pub mod stripped;
pub mod tag;
pub mod typing;

pub use algorithm::Algorithm;
pub use event_type::EventType;

/// An event that is malformed or otherwise invalid.
///
/// When attempting to deserialize an [`EventResult`](enum.EventResult.html), an error in the input
/// data may cause deserialization to fail, or the JSON structure may be correct, but additional
/// constraints defined in the matrix specification are not upheld. This type provides an error
/// message and a `serde_json::Value` representation of the invalid event, as well as a flag for
/// which type of error was encountered.
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
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Error for InvalidEvent {}

/// An error returned when attempting to create an event with data that would make it invalid.
///
/// This type is similar to [`InvalidEvent`](struct.InvalidEvent.html), but used during the
/// construction of a new event, as opposed to deserialization of an existing event from JSON.
#[derive(Clone, Debug, PartialEq)]
pub struct InvalidInput(String);

impl Display for InvalidInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl Error for InvalidInput {}

/// See [`TryFromRaw`][try]. This trait is merely a convenience that is be implemented instead of
/// [`TryFromRaw`][try] to get a [`TryFromRaw`][try] implementation with slightly less code if the
/// conversion can't fail, that is, the raw type and `Self` are identical in definition.
///
/// [try]: trait.TryFromRaw.html
pub trait FromRaw: Sized {
    /// The raw type.
    type Raw: DeserializeOwned;

    /// Converts the raw type to `Self`.
    fn from_raw(_: Self::Raw) -> Self;
}

/// Types corresponding to some item in the matrix spec. Types that implement this trait have a
/// corresponding 'raw' type, a potentially invalid representation that can be converted to `Self`.
pub trait TryFromRaw: Sized {
    /// The raw type.
    type Raw: DeserializeOwned;
    /// The error type returned if conversion fails.
    type Err: Display;

    /// Tries to convert the raw type to `Self`.
    fn try_from_raw(_: Self::Raw) -> Result<Self, Self::Err>;
}

impl<T: FromRaw> TryFromRaw for T {
    type Raw = <T as FromRaw>::Raw;
    type Err = Infallible;

    fn try_from_raw(raw: Self::Raw) -> Result<Self, Self::Err> {
        Ok(Self::from_raw(raw))
    }
}

/// The result of deserializing an event, which may or may not be valid.
///
/// When data is successfully deserialized and validated, this structure will contain the
/// deserialized value `T`. When deserialization succeeds, but the event is invalid for any reason,
/// this structure will contain an [`InvalidEvent`](struct.InvalidEvent.html). See the documentation
/// for [`InvalidEvent`](struct.InvalidEvent.html) for more details.
#[derive(Clone, Debug)]
pub enum EventResult<T: TryFromRaw> {
    /// `T` deserialized and validated successfully.
    Ok(T),

    /// `T` failed either deserialization or validation.
    ///
    /// [`InvalidEvent`](struct.InvalidEvent.html) contains the error message and the raw data.
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

/// An error when attempting to create a value from a string via the `FromStr` trait.
#[derive(Clone, Copy, Eq, Debug, Hash, PartialEq)]
pub struct FromStrError;

impl Display for FromStrError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "failed to parse type from string")
    }
}

impl Error for FromStrError {}

/// A meaningless value that serializes to an empty JSON object.
///
/// This type is used in a few places where the Matrix specification requires an empty JSON object,
/// but it's wasteful to represent it as a `HashMap` in Rust code.
#[derive(Clone, Debug, PartialEq)]
pub struct Empty;

impl Serialize for Empty {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_map(Some(0))?.end()
    }
}

impl<'de> Deserialize<'de> for Empty {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EmptyMapVisitor;

        impl<'de> Visitor<'de> for EmptyMapVisitor {
            type Value = Empty;

            fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
                write!(f, "an object/map")
            }

            fn visit_map<A>(self, _map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                Ok(Empty)
            }
        }

        deserializer.deserialize_map(EmptyMapVisitor)
    }
}

/// A basic event.
pub trait Event: Debug + Serialize + Sized + TryFromRaw {
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
    fn event_id(&self) -> &EventId;

    /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver when this event was
    /// sent.
    fn origin_server_ts(&self) -> UInt;

    /// The unique identifier for the room associated with this event.
    ///
    /// This can be `None` if the event came from a context where there is
    /// no ambiguity which room it belongs to, like a `/sync` response for example.
    fn room_id(&self) -> Option<&RoomId>;

    /// The unique identifier for the user who sent this event.
    fn sender(&self) -> &UserId;

    /// Additional key-value pairs not signed by the homeserver.
    fn unsigned(&self) -> Option<&Value>;
}

/// An event that describes persistent state about a room.
pub trait StateEvent: RoomEvent {
    /// The previous content for this state key, if any.
    fn prev_content(&self) -> Option<&Self::Content>;

    /// A key that determines which piece of room state the event represents.
    fn state_key(&self) -> &str;
}

mod custom {
    use ruma_events_macros::ruma_event;
    use serde_json::Value;

    ruma_event! {
        /// A custom basic event not covered by the Matrix specification.
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

mod custom_room {
    use ruma_events_macros::ruma_event;
    use serde_json::Value;

    ruma_event! {
        /// A custom room event not covered by the Matrix specification.
        CustomRoomEvent {
            kind: RoomEvent,
            event_type: Custom,
            content_type_alias: {
                /// The payload for `CustomRoomEvent`.
                Value
            },
        }
    }
}

mod custom_state {
    use ruma_events_macros::ruma_event;
    use serde_json::Value;

    ruma_event! {
        /// A custom state event not covered by the Matrix specification.
        CustomStateEvent {
            kind: StateEvent,
            event_type: Custom,
            content_type_alias: {
                /// The payload for `CustomStateEvent`.
                Value
            },
        }
    }
}
