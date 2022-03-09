//! (De)serializable types for the events in the [Matrix](https://matrix.org) specification.
//! These types are used by other Ruma crates.
//!
//! All data exchanged over Matrix is expressed as an event.
//! Different event types represent different actions, such as joining a room or sending a message.
//! Events are stored and transmitted as simple JSON structures.
//! While anyone can create a new event type for their own purposes, the Matrix specification
//! defines a number of event types which are considered core to the protocol, and Matrix clients
//! and servers must understand their semantics.
//! This module contains Rust types for each of the event types defined by the specification and
//! facilities for extending the event system for custom event types.
//!
//! # Event types
//!
//! This module includes a Rust enum called [`EventType`], which provides a simple enumeration of
//! all the event types defined by the Matrix specification. Matrix event types are serialized to
//! JSON strings in [reverse domain name
//! notation](https://en.wikipedia.org/wiki/Reverse_domain_name_notation), although the core event
//! types all use the special "m" TLD, e.g. `m.room.message`.
//!
//! # Core event types
//!
//! This module includes Rust types for every one of the event types in the Matrix specification.
//! To better organize the crate, these types live in separate modules with a hierarchy that
//! matches the reverse domain name notation of the event type.
//! For example, the `m.room.message` event lives at
//! `ruma_common::events::::room::message::MessageLikeEvent`. Each type's module also contains a
//! Rust type for that event type's `content` field, and any other supporting types required by the
//! event's other fields.
//!
//! # Extending Ruma with custom events
//!
//! For our examples we will start with a simple custom state event. `ruma_event`
//! specifies the state event's `type` and it's [`kind`](EventKind).
//!
//! ```rust
//! use ruma_common::events::macros::EventContent;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
//! #[ruma_event(type = "org.example.event", kind = State)]
//! pub struct ExampleContent {
//!     field: String,
//! }
//! ```
//!
//! This can be used with events structs, such as passing it into
//! `ruma::api::client::state::send_state_event`'s `Request`.
//!
//! As a more advanced example we create a reaction message event. For this event we will use a
//! [`SyncMessageLikeEvent`] struct but any [`MessageLikeEvent`] struct would work.
//!
//! ```rust
//! use ruma_common::events::{macros::EventContent, SyncMessageLikeEvent};
//! use ruma_common::EventId;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Clone, Debug, Deserialize, Serialize)]
//! #[serde(tag = "rel_type")]
//! pub enum RelatesTo {
//!     #[serde(rename = "m.annotation")]
//!     Annotation {
//!         /// The event this reaction relates to.
//!         event_id: Box<EventId>,
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
//! #[ruma_event(type = "m.reaction", kind = MessageLike)]
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
//!     serde_json::from_value::<SyncMessageLikeEvent<ReactionEventContent>>(json),
//!     Ok(SyncMessageLikeEvent {
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
//! All concrete event types in this module can be serialized via the `Serialize` trait from
//! [serde](https://serde.rs/) and can be deserialized from a `Raw<EventType>`. In order to
//! handle incoming data that may not conform to this module's strict definitions of event
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

use ruma_serde::Raw;
use serde::{de::IgnoredAny, Deserialize, Serialize, Serializer};
use serde_json::value::RawValue as RawJsonValue;

use self::room::redaction::SyncRoomRedactionEvent;
use crate::{EventEncryptionAlgorithm, RoomVersionId};

// Needs to be public for trybuild tests
#[doc(hidden)]
pub mod _custom;
mod enums;
mod event_kinds;
mod unsigned;

/// Re-export of all the derives needed to create your own event types.
pub mod macros {
    pub use ruma_macros::{Event, EventContent};
}

pub mod call;
pub mod direct;
pub mod dummy;
#[cfg(feature = "unstable-msc1767")]
pub mod emote;
#[cfg(feature = "unstable-msc3551")]
pub mod file;
pub mod forwarded_room_key;
pub mod fully_read;
pub mod ignored_user_list;
pub mod key;
#[cfg(feature = "unstable-msc1767")]
pub mod message;
#[cfg(feature = "unstable-msc1767")]
pub mod notice;
#[cfg(feature = "unstable-pdu")]
pub mod pdu;
pub mod policy;
pub mod presence;
pub mod push_rules;
#[cfg(feature = "unstable-msc2677")]
pub mod reaction;
pub mod receipt;
#[cfg(feature = "unstable-msc2675")]
pub mod relation;
pub mod room;
pub mod room_key;
pub mod room_key_request;
pub mod secret;
pub mod space;
pub mod sticker;
pub mod tag;
pub mod typing;

#[cfg(feature = "unstable-msc2675")]
pub use self::relation::Relations;
#[doc(hidden)]
#[cfg(feature = "compat")]
pub use self::unsigned::{RedactedUnsignedWithPrevContent, UnsignedWithPrevContent};
pub use self::{
    enums::*,
    event_kinds::*,
    unsigned::{RedactedUnsigned, Unsigned},
};

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
    fn redact(self, redaction: SyncRoomRedactionEvent, version: &RoomVersionId) -> Self::Redacted;
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

/// Marker trait for the content of a message-like event.
pub trait MessageLikeEventContent: EventContent {}

/// Marker trait for the content of a state event.
pub trait StateEventContent: EventContent {}

/// The base trait that all redacted event content types implement.
///
/// This trait's associated functions and methods should not be used to build
/// redacted events, prefer the `redact` method on `AnyStateEvent` and
/// `AnyMessageLikeEvent` and their "sync" and "stripped" counterparts. The
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

/// Marker trait for the content of a redacted message-like event.
pub trait RedactedMessageLikeEventContent: RedactedEventContent {}

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

    /// Message-like event kind.
    ///
    /// Since redacted / non-redacted message-like events are used in the same places but have
    /// different sets of fields, these two variations are treated as two closely-related event
    /// kinds.
    MessageLike {
        /// Redacted variation?
        redacted: bool,
    },

    /// State event kind.
    ///
    /// Since redacted / non-redacted state events are used in the same places but have different
    /// sets of fields, these two variations are treated as two closely-related event kinds.
    State {
        /// Redacted variation?
        redacted: bool,
    },

    /// To-device event kind.
    ToDevice,

    /// Presence event kind.
    Presence,

    /// Hierarchy space child kind.
    HierarchySpaceChild,
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

/// Helper struct to determine the event kind from a `serde_json::value::RawValue`.
#[doc(hidden)]
#[derive(Deserialize)]
#[allow(clippy::exhaustive_structs)]
pub struct EventTypeDeHelper<'a> {
    #[serde(borrow, rename = "type")]
    pub ev_type: std::borrow::Cow<'a, str>,
}

/// Helper struct to determine if an event has been redacted.
#[doc(hidden)]
#[derive(Deserialize)]
#[allow(clippy::exhaustive_structs)]
pub struct RedactionDeHelper {
    /// Used to check whether redacted_because exists.
    pub unsigned: Option<UnsignedDeHelper>,
}

#[doc(hidden)]
#[derive(Deserialize)]
#[allow(clippy::exhaustive_structs)]
pub struct UnsignedDeHelper {
    /// This is the field that signals an event has been redacted.
    pub redacted_because: Option<IgnoredAny>,
}

/// Helper function for erroring when trying to serialize an event enum _Custom variant that can
/// only be created by deserializing from an unknown event type.
#[doc(hidden)]
#[allow(clippy::ptr_arg)]
pub fn serialize_custom_event_error<T, S: Serializer>(_: &T, _: S) -> Result<S::Ok, S::Error> {
    Err(serde::ser::Error::custom(
        "Failed to serialize event [content] enum: Unknown event type.\n\
         To send custom events, turn them into `Raw<EnumType>` by going through
         `serde_json::value::to_raw_value` and `Raw::from_json`.",
    ))
}
