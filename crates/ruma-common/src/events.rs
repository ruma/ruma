//! (De)serializable types for the events in the [Matrix](https://matrix.org) specification.
//! These types are used by other Ruma crates.
//!
//! All data exchanged over Matrix is expressed as an event.
//! Different event types represent different actions, such as joining a room or sending a message.
//! Events are stored and transmitted as simple JSON structures.
//! While anyone can create a new event type for their own purposes, the Matrix specification
//! defines a number of event types which are considered core to the protocol.
//! This module contains Rust types for all of the event types defined by the specification and
//! facilities for extending the event system for custom event types.
//!
//! # Core event types
//!
//! This module includes Rust types for all event types in the Matrix specification.
//! To better organize the crate, these types live in separate modules with a hierarchy that matches
//! the reverse domain name notation of the event type. For example, the `m.room.message` event
//! lives at `ruma::events::room::message::RoomMessageEvent`. Each type's module also contains a
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
//! #[ruma_event(type = "org.example.event", kind = State, state_key_type = String)]
//! pub struct ExampleContent {
//!     field: String,
//! }
//! ```
//!
//! This can be used with events structs, such as passing it into
//! `ruma::api::client::state::send_state_event`'s `Request`.
//!
//! As a more advanced example we create a reaction message event. For this event we will use a
//! [`OriginalSyncMessageLikeEvent`] struct but any [`OriginalMessageLikeEvent`] struct would work.
//!
//! ```rust
//! use ruma_common::{
//!     events::{macros::EventContent, OriginalSyncMessageLikeEvent},
//!     OwnedEventId,
//! };
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Clone, Debug, Deserialize, Serialize)]
//! #[serde(tag = "rel_type")]
//! pub enum RelatesTo {
//!     #[serde(rename = "m.annotation")]
//!     Annotation {
//!         /// The event this reaction relates to.
//!         event_id: OwnedEventId,
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
//! assert_matches::assert_matches!(
//!     serde_json::from_value::<OriginalSyncMessageLikeEvent<ReactionEventContent>>(json),
//!     Ok(OriginalSyncMessageLikeEvent {
//!         content: ReactionEventContent {
//!             relates_to: RelatesTo::Annotation { key, .. },
//!         },
//!         ..
//!     }) if key == "👍"
//! );
//! ```

use serde::{de::IgnoredAny, Deserialize, Serializer};

use self::room::redaction::SyncRoomRedactionEvent;
use crate::{EventEncryptionAlgorithm, RoomVersionId};

// Needs to be public for trybuild tests
#[doc(hidden)]
pub mod _custom;
mod content;
mod enums;
mod kinds;
mod state_key;
mod unsigned;

/// Re-export of all the derives needed to create your own event types.
pub mod macros {
    pub use ruma_macros::{Event, EventContent};
}

#[cfg(feature = "unstable-msc3246")]
pub mod audio;
pub mod call;
pub mod direct;
pub mod dummy;
#[cfg(feature = "unstable-msc1767")]
pub mod emote;
#[cfg(feature = "unstable-msc3551")]
pub mod file;
pub mod forwarded_room_key;
pub mod fully_read;
pub mod identity_server;
pub mod ignored_user_list;
#[cfg(feature = "unstable-msc3552")]
pub mod image;
pub mod key;
#[cfg(feature = "unstable-msc3488")]
pub mod location;
#[cfg(feature = "unstable-msc1767")]
pub mod message;
#[cfg(feature = "unstable-msc1767")]
pub mod notice;
#[cfg(feature = "unstable-pdu")]
pub mod pdu;
pub mod policy;
#[cfg(feature = "unstable-msc3381")]
pub mod poll;
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
pub mod secret_storage;
pub mod space;
pub mod sticker;
pub mod tag;
pub mod typing;
#[cfg(feature = "unstable-msc3553")]
pub mod video;
#[cfg(feature = "unstable-msc3245")]
pub mod voice;

#[cfg(feature = "unstable-msc2675")]
pub use self::relation::Relations;
pub use self::{
    content::*,
    enums::*,
    kinds::*,
    state_key::EmptyStateKey,
    unsigned::{MessageLikeUnsigned, RedactedUnsigned, StateUnsigned},
};

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
