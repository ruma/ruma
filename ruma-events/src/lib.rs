//! Crate `ruma_events` contains serializable types for the events in the
//! [Matrix](https://matrix.org) specification that can be shared by client and server code.
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
//!     * `type`, which specifies the event's type
//!     * `content`, which is a JSON object containing the "payload" of the event
//! 2.  **Room events**, which are a superset of events and represent actions that occurred within
//!     the context of a Matrix room.
//!     They have at least the following additional keys:
//!     * `event_id`, which is a unique identifier for the event
//!     * `room_id`, which is a unique identifier for the room in which the event occurred
//!     * `sender`, which is the unique identifier of the Matrix user who created the event
//!     * Optionally, `unsigned`, which is a JSON object containing arbitrary additional metadata
//!     that is not digitally signed by Matrix homeservers.
//! 3.  **State events**, which are a superset of room events and represent persistent state
//!     specific to a room, such as the room's member list or topic.
//!     Within a single room, state events of the same type and with the same "state key" will
//!     effectively "replace" the previous one, updating the room's state.
//!     They have at least the following additional keys:
//!     * `state_key`, a string which serves as a sort of "sub-type." The state key allows a room to
//!       persist multiple state events of the same type. You can think of a room's state events as
//!       being a `BTreeMap` where the keys are the tuple `(event_type, state_key)`.
//!     * Optionally, `prev_content`, a JSON object containing the `content` object from the
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
//! ## Extending Ruma with custom events
//!
//! For our example we will create a reaction message event. This can be used with ruma-events
//! structs, for this event we will use a `SyncMessageEvent` struct but any `MessageEvent` struct
//! would work.
//!
//! ```rust
//! use ruma_events::{macros::MessageEventContent, SyncMessageEvent};
//! use ruma_identifiers::EventId;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Clone, Debug, Deserialize, Serialize)]
//! #[serde(tag = "rel_type")]
//! pub enum RelatesTo {
//!     #[serde(rename = "m.annotation")]
//!     Annotation {
//!         /// The event this reaction relates to.
//!         event_id: EventId,
//!         /// The displayable content of the reaction.
//!         key: String,
//!     },
//!
//!     /// Since this event is not fully specified in the Matrix spec
//!     /// it may change or types may be added, we are ready!
//!     #[serde(rename = "m.whatever")]
//!     Whatever,
//! }
//!
//! /// The payload for our reaction event.
//! #[derive(Clone, Debug, Deserialize, Serialize, MessageEventContent)]
//! #[ruma_event(type = "m.reaction")]
//! pub struct ReactionEventContent {
//!     #[serde(rename = "m.relates_to")]
//!     pub relates_to: RelatesTo,
//! }
//!
//! let json = serde_json::json!({
//!     "content": {
//!         "m.relates_to": {
//!             "event_id": "$xxxx-xxxx",
//!             "key": "👍",
//!             "rel_type": "m.annotation"
//!         }
//!     },
//!     "event_id": "$xxxx-xxxx",
//!     "origin_server_ts": 1,
//!     "sender": "@someone:example.org",
//!     "type": "m.reaction",
//!     "unsigned": {
//!         "age": 85
//!     }
//! });
//!
//! // The downside of this event is we cannot use it with the `AnyRoomEvent` or `AnyEvent` enums,
//! // but could be deserialized from a `Raw<AnyRoomEvent>` that has failed.
//! matches::assert_matches!(
//!     serde_json::from_value::<SyncMessageEvent<ReactionEventContent>>(json),
//!     Ok(SyncMessageEvent {
//!         content: ReactionEventContent {
//!             relates_to: RelatesTo::Annotation { key, .. },
//!         },
//!         ..
//!     }) if key == "👍"
//! );
//! ```
//!
//! # Serialization and deserialization
//!
//! All concrete event types in ruma-events can be serialized via the `Serialize` trait from
//! [serde](https://serde.rs/) and can be deserialized from as `Raw<EventType>`. In order to
//! handle incoming data that may not conform to `ruma-events`' strict definitions of event
//! structures, deserialization will return `Raw::Err` on error. This error covers both
//! structurally invalid JSON data as well as structurally valid JSON that doesn't fulfill
//! additional constraints the matrix specification defines for some event types. The error exposes
//! the deserialized `serde_json::Value` so that developers can still work with the received
//! event data. This makes it possible to deserialize a collection of events without the entire
//! collection failing to deserialize due to a single invalid event. The "content" type for each
//! event also implements `Serialize` and either `TryFromRaw` (enabling usage as
//! `Raw<ContentType>` for dedicated content types) or `Deserialize` (when the content is a
//! type alias), allowing content to be converted to and from JSON independently of the surrounding
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

#![recursion_limit = "1024"]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
// This lint is no good
#![allow(clippy::new_without_default)]
// Remove this once https://github.com/rust-lang/rust/issues/54883 becomes stable
#![allow(clippy::unnested_or_patterns)]
#![allow(clippy::unknown_clippy_lints)]

use std::fmt::Debug;

use js_int::Int;
use ruma_identifiers::{EventEncryptionAlgorithm, RoomId};
use ruma_serde::Raw;
use serde::{
    de::{self, IgnoredAny},
    Deserialize, Serialize,
};
use serde_json::value::RawValue as RawJsonValue;

use self::room::redaction::{RedactionEvent, SyncRedactionEvent};

mod enums;
mod error;
mod event_kinds;
mod event_type;

// Hack to allow both ruma-events itself and external crates (or tests) to use procedural macros
// that expect `ruma_events` to exist in the prelude.
extern crate self as ruma_events;

/// Re-exports to allow users to declare their own event types using the
/// macros used internally.
///
/// It is not considered part of ruma-events' public API.
#[doc(hidden)]
pub mod exports {
    pub use js_int;
    pub use ruma_identifiers;
    pub use serde;
    pub use serde_json;
}

/// Re-export of all the derives needed to create your own event types.
pub mod macros {
    pub use ruma_events_macros::{
        BasicEventContent, EphemeralRoomEventContent, Event, MessageEventContent, StateEventContent,
    };
}

pub mod call;
pub mod custom;
pub mod direct;
pub mod dummy;
pub mod forwarded_room_key;
pub mod fully_read;
pub mod ignored_user_list;
pub mod key;
pub mod pdu;
pub mod policy;
pub mod presence;
pub mod push_rules;
#[cfg(feature = "unstable-pre-spec")]
pub mod reaction;
pub mod receipt;
#[cfg(feature = "unstable-pre-spec")]
pub mod relation;
pub mod room;
pub mod room_key;
pub mod room_key_request;
pub mod sticker;
pub mod tag;
pub mod typing;

#[cfg(feature = "unstable-pre-spec")]
pub use self::relation::Relations;
pub use self::{
    enums::{
        AnyBasicEvent, AnyBasicEventContent, AnyEphemeralRoomEvent, AnyEphemeralRoomEventContent,
        AnyEvent, AnyInitialStateEvent, AnyMessageEvent, AnyMessageEventContent,
        AnyPossiblyRedactedMessageEvent, AnyPossiblyRedactedStateEvent,
        AnyPossiblyRedactedStrippedStateEvent, AnyPossiblyRedactedSyncMessageEvent,
        AnyPossiblyRedactedSyncStateEvent, AnyRedactedMessageEvent, AnyRedactedStateEvent,
        AnyRedactedStrippedStateEvent, AnyRedactedSyncMessageEvent, AnyRedactedSyncStateEvent,
        AnyRoomEvent, AnyStateEvent, AnyStateEventContent, AnyStrippedStateEvent,
        AnySyncEphemeralRoomEvent, AnySyncMessageEvent, AnySyncRoomEvent, AnySyncStateEvent,
        AnyToDeviceEvent, AnyToDeviceEventContent,
    },
    error::{FromStrError, InvalidInput},
    event_kinds::{
        BasicEvent, EphemeralRoomEvent, InitialStateEvent, MessageEvent, RedactedMessageEvent,
        RedactedStateEvent, RedactedStrippedStateEvent, RedactedSyncMessageEvent,
        RedactedSyncStateEvent, StateEvent, StrippedStateEvent, SyncEphemeralRoomEvent,
        SyncMessageEvent, SyncStateEvent, ToDeviceEvent,
    },
    event_type::EventType,
};

/// Extra information about an event that is not incorporated into the event's
/// hash.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Unsigned {
    /// The time in milliseconds that has elapsed since the event was sent. This
    /// field is generated by the local homeserver, and may be incorrect if the
    /// local time on at least one of the two servers is out of sync, which can
    /// cause the age to either be negative or greater than it actually is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<Int>,

    /// The client-supplied transaction ID, if the client being given the event
    /// is the same one which sent it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,

    /// Server-compiled information from other events relating to this event.
    #[cfg(feature = "unstable-pre-spec")]
    #[serde(rename = "m.relations", skip_serializing_if = "Option::is_none")]
    pub relations: Option<Relations>,
}

impl Unsigned {
    /// Whether this unsigned data is empty (all fields are `None`).
    ///
    /// This method is used to determine whether to skip serializing the
    /// `unsigned` field in room events. Do not use it to determine whether
    /// an incoming `unsigned` field was present - it could still have been
    /// present but contained none of the known fields.
    pub fn is_empty(&self) -> bool {
        self.age.is_none() && self.transaction_id.is_none()
    }
}

/// Extra information about a redacted event that is not incorporated into the event's
/// hash.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RedactedUnsigned {
    /// The event that redacted this event, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redacted_because: Option<Box<RedactionEvent>>,
}

impl RedactedUnsigned {
    /// Whether this unsigned data is empty (`redacted_because` is `None`).
    ///
    /// This method is used to determine whether to skip serializing the
    /// `unsigned` field in redacted room events. Do not use it to determine whether
    /// an incoming `unsigned` field was present - it could still have been
    /// present but contained none of the known fields.
    pub fn is_empty(&self) -> bool {
        self.redacted_because.is_none()
    }
}

/// Extra information about a redacted sync event that is not incorporated into the sync event's
/// hash.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RedactedSyncUnsigned {
    /// The event that redacted this event, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redacted_because: Option<Box<SyncRedactionEvent>>,
}

impl From<RedactedUnsigned> for RedactedSyncUnsigned {
    fn from(redacted: RedactedUnsigned) -> Self {
        match redacted.redacted_because.map(|b| *b) {
            Some(RedactionEvent {
                sender,
                event_id,
                origin_server_ts,
                redacts,
                unsigned,
                content,
                ..
            }) => Self {
                redacted_because: Some(Box::new(SyncRedactionEvent {
                    sender,
                    event_id,
                    origin_server_ts,
                    redacts,
                    unsigned,
                    content,
                })),
            },
            _ => Self { redacted_because: None },
        }
    }
}

impl RedactedSyncUnsigned {
    /// Whether this unsigned data is empty (`redacted_because` is `None`).
    ///
    /// This method is used to determine whether to skip serializing the
    /// `unsignedSync` field in redacted room events. Do not use it to determine whether
    /// an incoming `unsignedSync` field was present - it could still have been
    /// present but contained none of the known fields.
    pub fn is_empty(&self) -> bool {
        self.redacted_because.is_none()
    }

    /// Convert a `RedactedSyncUnsigned` into `RedactedUnsigned`, converting the
    /// underlying sync redaction event to a full redaction event (with room_id).
    pub fn into_full(self, room_id: RoomId) -> RedactedUnsigned {
        match self.redacted_because.map(|b| *b) {
            Some(SyncRedactionEvent {
                sender,
                event_id,
                origin_server_ts,
                redacts,
                unsigned,
                content,
            }) => RedactedUnsigned {
                redacted_because: Some(Box::new(RedactionEvent {
                    room_id,
                    sender,
                    event_id,
                    origin_server_ts,
                    redacts,
                    unsigned,
                    content,
                })),
            },
            _ => RedactedUnsigned { redacted_because: None },
        }
    }
}

/// The base trait that all event content types implement.
///
/// Implementing this trait allows content types to be serialized as well as deserialized.
pub trait EventContent: Sized + Serialize {
    /// A matrix event identifier, like `m.room.message`.
    fn event_type(&self) -> &str;

    /// Constructs the given event content.
    fn from_parts(event_type: &str, content: Box<RawJsonValue>) -> Result<Self, serde_json::Error>;
}

/// Extension trait for Raw<EventContent>
pub trait RawExt<T: EventContent> {
    /// Try to deserialize the JSON as event content
    fn deserialize_content(self, event_type: &str) -> Result<T, serde_json::Error>;
}

impl<T: EventContent> RawExt<T> for Raw<T>
where
    T: EventContent,
{
    /// Try to deserialize the JSON as event content
    fn deserialize_content(self, event_type: &str) -> Result<T, serde_json::Error> {
        T::from_parts(event_type, self.into_json())
    }
}

/// Marker trait for the content of an ephemeral room event.
pub trait EphemeralRoomEventContent: EventContent {}

/// Marker trait for the content of a basic event.
pub trait BasicEventContent: EventContent {}

/// Marker trait for the content of a room event.
pub trait RoomEventContent: EventContent {}

/// Marker trait for the content of a message event.
pub trait MessageEventContent: RoomEventContent {}

/// Marker trait for the content of a state event.
pub trait StateEventContent: RoomEventContent {}

/// The base trait that all redacted event content types implement.
///
/// This trait's associated functions and methods should not be used to build
/// redacted events, prefer the `redact` method on `AnyStateEvent` and
/// `AnyMessageEvent` and their "sync" and "stripped" counterparts. The
/// `RedactedEventContent` trait is an implementation detail, ruma makes no
/// API guarantees.
pub trait RedactedEventContent: EventContent {
    /// Constructs the redacted event content.
    ///
    /// If called for anything but "empty" redacted content this will error.
    #[doc(hidden)]
    fn empty(_event_type: &str) -> Result<Self, serde_json::Error> {
        Err(serde::de::Error::custom("this event is not redacted"))
    }

    /// Determines if the redacted event content needs to serialize fields.
    #[doc(hidden)]
    fn has_serialize_fields(&self) -> bool;

    /// Determines if the redacted event content needs to deserialize fields.
    #[doc(hidden)]
    fn has_deserialize_fields() -> HasDeserializeFields;
}

/// Marker trait for the content of a redacted message event.
pub trait RedactedMessageEventContent: RedactedEventContent {}

/// Marker trait for the content of a redacted state event.
pub trait RedactedStateEventContent: RedactedEventContent {}

/// `HasDeserializeFields` is used in the code generated by the `Event` derive
/// to aid in deserializing redacted events.
#[doc(hidden)]
#[derive(Debug)]
pub enum HasDeserializeFields {
    /// Deserialize the event's content, failing if invalid.
    True,

    /// Return the redacted version of this event's content.
    False,

    /// `Optional` is used for `RedactedAliasesEventContent` since it has
    /// an empty version and one with content left after redaction that
    /// must be supported together.
    Optional,
}

/// Helper struct to determine if the event has been redacted.
#[doc(hidden)]
#[derive(Debug, Deserialize)]
pub struct UnsignedDeHelper {
    /// This is the field that signals an event has been redacted.
    pub redacted_because: Option<IgnoredAny>,
}

/// Helper struct to determine the event kind from a serde_json::value::RawValue.
#[doc(hidden)]
#[derive(Debug, Deserialize)]
pub struct EventDeHelper {
    /// the Matrix event type string "m.room.whatever".
    #[serde(rename = "type")]
    pub ev_type: String,

    /// If `state_key` is present the event will be deserialized as a state event.
    pub state_key: Option<IgnoredAny>,

    /// If no `state_key` is found but an `event_id` is present the event
    /// will be deserialized as a message event.
    pub event_id: Option<IgnoredAny>,

    /// If no `event_id` or `state_key` are found but a `room_id` is present
    /// the event will be deserialized as a ephemeral event.
    pub room_id: Option<IgnoredAny>,

    /// If this `UnsignedData` contains a redacted_because key the event is
    /// immediately deserialized as a redacted event.
    pub unsigned: Option<UnsignedDeHelper>,
}

/// Helper function for serde_json::value::RawValue deserialization.
#[doc(hidden)]
pub fn from_raw_json_value<T, E>(val: &RawJsonValue) -> Result<T, E>
where
    T: de::DeserializeOwned,
    E: de::Error,
{
    serde_json::from_str(val.get()).map_err(E::custom)
}
