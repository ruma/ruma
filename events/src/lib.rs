#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
//! (De)serializable types for the events in the [Matrix](https://matrix.org) specification.
//! These types are used by other ruma crates.
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
//! # Extending Ruma with custom events
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

#![recursion_limit = "1024"]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]
// This lint is no good
#![allow(clippy::new_without_default)]
// Remove this once our MSRV is >= 1.53
#![allow(clippy::unnested_or_patterns)]

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
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
pub mod reaction;
pub mod receipt;
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
pub mod relation;
pub mod room;
pub mod room_key;
pub mod room_key_request;
pub mod sticker;
pub mod tag;
pub mod typing;

#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
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
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
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
                content,
                redacts,
                event_id,
                sender,
                origin_server_ts,
                unsigned,
                ..
            }) => Self {
                redacted_because: Some(Box::new(SyncRedactionEvent {
                    content,
                    redacts,
                    event_id,
                    sender,
                    origin_server_ts,
                    unsigned,
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
                content,
                redacts,
                event_id,
                sender,
                origin_server_ts,
                unsigned,
            }) => RedactedUnsigned {
                redacted_because: Some(Box::new(RedactionEvent {
                    content,
                    redacts,
                    event_id,
                    sender,
                    origin_server_ts,
                    room_id,
                    unsigned,
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

/// Helper struct to determine the event kind from a `serde_json::value::RawValue`.
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
    /// the event will be deserialized as an ephemeral event.
    pub room_id: Option<IgnoredAny>,

    /// If this `UnsignedData` contains a `redacted_because` key the event is
    /// immediately deserialized as a redacted event.
    pub unsigned: Option<UnsignedDeHelper>,
}

/// Helper function for `serde_json::value::RawValue` deserialization.
#[doc(hidden)]
pub fn from_raw_json_value<T, E>(val: &RawJsonValue) -> Result<T, E>
where
    T: de::DeserializeOwned,
    E: de::Error,
{
    serde_json::from_str(val.get()).map_err(E::custom)
}
