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
//! use ruma_events::{macros::EventContent, SyncMessageEvent};
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
//! #[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
//! #[ruma_event(type = "m.reaction", kind = Message)]
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
//! // The downside of this event is we cannot use it with event enums,
//! // but could be deserialized from a `Raw<_>` that has failed to deserialize.
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
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::fmt::Debug;

use js_int::Int;
use ruma_identifiers::{EventEncryptionAlgorithm, RoomVersionId};
use ruma_serde::Raw;
use serde::{
    de::{self, IgnoredAny},
    Deserialize, Serialize,
};
use serde_json::value::RawValue as RawJsonValue;

use self::room::redaction::SyncRedactionEvent;

mod enums;
mod event_kinds;

// Hack to allow both ruma-events itself and external crates (or tests) to use procedural macros
// that expect `ruma_events` to exist in the prelude.
extern crate self as ruma_events;

/// Re-exports to allow users to declare their own event types using the
/// macros used internally.
///
/// It is not considered part of ruma-events' public API.
#[doc(hidden)]
pub mod exports {
    pub use ruma_common;
    pub use ruma_identifiers;
    pub use ruma_serde;
    pub use serde;
    pub use serde_json;
}

/// Re-export of all the derives needed to create your own event types.
pub mod macros {
    pub use ruma_events_macros::{Event, EventContent};
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
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
pub mod secret;
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
pub mod space;
pub mod sticker;
pub mod tag;
pub mod typing;

#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
pub use self::relation::Relations;
pub use self::{enums::*, event_kinds::*};

/// Extra information about an event that is not incorporated into the event's
/// hash.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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
    /// Create a new `Unsigned` with fields set to `None`.
    pub fn new() -> Self {
        Self::default()
    }

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
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RedactedUnsigned {
    /// The event that redacted this event, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redacted_because: Option<Box<SyncRedactionEvent>>,
}

impl RedactedUnsigned {
    /// Create a new `RedactedUnsigned` with field set to `None`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new `RedactedUnsigned` with the given redacted because.
    pub fn new_because(redacted_because: Box<SyncRedactionEvent>) -> Self {
        Self { redacted_because: Some(redacted_because) }
    }

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

/// The base trait that all event content types implement.
///
/// Implementing this trait allows content types to be serialized as well as deserialized.
pub trait EventContent: Sized + Serialize {
    /// A matrix event identifier, like `m.room.message`.
    fn event_type(&self) -> &str;

    /// Constructs the given event content.
    fn from_parts(event_type: &str, content: &RawJsonValue) -> serde_json::Result<Self>;
}

/// Trait to define the behavior of redacting an event.
pub trait Redact {
    /// The redacted form of the event.
    type Redacted;

    /// Transforms `self` into a redacted form (removing most fields) according to the spec.
    ///
    /// A small number of events have room-version specific redaction behavior, so a version has to
    /// be specified.
    fn redact(self, redaction: SyncRedactionEvent, version: &RoomVersionId) -> Self::Redacted;
}

/// Trait to define the behavior of redact an event's content object.
pub trait RedactContent {
    /// The redacted form of the event's content.
    type Redacted;

    /// Transform `self` into a redacted form (removing most or all fields) according to the spec.
    ///
    /// A small number of events have room-version specific redaction behavior, so a version has to
    /// be specified.
    ///
    /// Where applicable, it is preferred to use [`Redact::redact`] on the outer event.
    fn redact(self, version: &RoomVersionId) -> Self::Redacted;
}

/// Extension trait for [`Raw<_>`][ruma_serde::Raw].
pub trait RawExt<T: EventContent> {
    /// Try to deserialize the JSON as an event's content.
    fn deserialize_content(&self, event_type: &str) -> serde_json::Result<T>;
}

impl<T: EventContent> RawExt<T> for Raw<T> {
    fn deserialize_content(&self, event_type: &str) -> serde_json::Result<T> {
        T::from_parts(event_type, self.json())
    }
}

/// Marker trait for the content of an ephemeral room event.
pub trait EphemeralRoomEventContent: EventContent {}

/// Marker trait for the content of a global account data event.
pub trait GlobalAccountDataEventContent: EventContent {}

/// Marker trait for the content of a room account data event.
pub trait RoomAccountDataEventContent: EventContent {}

/// Marker trait for the content of a to device event.
pub trait ToDeviceEventContent: EventContent {}

/// Marker trait for the content of a message event.
pub trait MessageEventContent: EventContent {}

/// Marker trait for the content of a state event.
pub trait StateEventContent: EventContent {}

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
    fn empty(_event_type: &str) -> serde_json::Result<Self> {
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

/// Trait for abstracting over event content structs.
///
/// … but *not* enums which don't always have an event type and kind (e.g. message vs state) that's
/// fixed / known at compile time.
pub trait StaticEventContent: EventContent {
    /// The event's "kind".
    ///
    /// See the type's documentation.
    const KIND: EventKind;

    /// The event type.
    const TYPE: &'static str;
}

/// The "kind" of an event.
///
/// This corresponds directly to the event content marker traits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum EventKind {
    /// Global account data event kind.
    GlobalAccountData,

    /// Room account data event kind.
    RoomAccountData,

    /// Ephemeral room event kind.
    EphemeralRoomData,

    /// Message event kind.
    ///
    /// Since redacted / non-redacted message events are used in the same places bu have different
    /// sets of fields, these two variations are treated as two closely-related event kinds.
    Message {
        /// Redacted variation?
        redacted: bool,
    },

    /// State event kind.
    ///
    /// Since redacted / non-redacted state events are used in the same places bu have different
    /// sets of fields, these two variations are treated as two closely-related event kinds.
    State {
        /// Redacted variation?
        redacted: bool,
    },

    /// To-device event kind.
    ToDevice,

    /// Presence event kind.
    Presence,
}

/// `HasDeserializeFields` is used in the code generated by the `Event` derive
/// to aid in deserializing redacted events.
#[doc(hidden)]
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
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
#[allow(clippy::exhaustive_structs)]
pub struct UnsignedDeHelper {
    /// This is the field that signals an event has been redacted.
    pub redacted_because: Option<IgnoredAny>,
}

/// Helper struct to determine the event kind from a `serde_json::value::RawValue`.
#[doc(hidden)]
#[derive(Debug, Deserialize)]
#[allow(clippy::exhaustive_structs)]
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
