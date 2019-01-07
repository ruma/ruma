//! Crate ruma_events contains serializable types for the events in the [Matrix](https://matrix.org)
//! specification that can be shared by client and server code.
//!
//! All data exchanged over Matrix is expressed as an event.
//! Different event types represent different actions, such as joining a room or sending a message.
//! Events are stored and transmitted as simple JSON structures.
//! While anyone can create a new event type for their own purposes, the Matrix specification
//! defines a number of event types which are considered core to the protocol, and Matrix clients
//! and servers must understand their semantics.
//! ruma_events contains Rust types for each of the event types defined by the specification and
//! facilities for extending the event system for custom event types.
//!
//! # Event types
//!
//! ruma_events includes a Rust enum called `EventType`, which provides a simple enumeration of
//! all the event types defined by the Matrix specification. Matrix event types are serialized to
//! JSON strings in [reverse domain name
//! notation](https://en.wikipedia.org/wiki/Reverse_domain_name_notation), although the core event
//! types all use the special "m" TLD, e.g. *m.room.message*.
//! `EventType` also includes a variant called `Custom`, which is a catch-all that stores a string
//! containing the name of any event type that isn't part of the specification.
//! `EventType` is used throughout ruma_events to identify and differentiate between events of
//! different types.
//!
//! # Event traits
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
//! ruma_events represents these three event kinds as traits, allowing any Rust type to serve as a
//! Matrix event so long as it upholds the contract expected of its kind.
//!
//! # Core event types
//!
//! ruma_events includes Rust types for every one of the event types in the Matrix specification.
//! To better organize the crate, these types live in separate modules with a hierarchy that
//! matches the reverse domain name notation of the event type.
//! For example, the *m.room.message* event lives at `ruma_events::room::message::MessageEvent`.
//! Each type's module also contains a Rust type for that event type's `content` field, and any
//! other supporting types required by the event's other fields.
//! All concrete event types in ruma_events are serializable and deserializable using the
//! [Serde](https://serde.rs/) serialization library.
//!
//! # Custom events
//!
//! Although any Rust type that implements `Event`, `RoomEvent`, or `StateEvent` can serve as a
//! Matrix event type, ruma_events also includes a few convenience types for representing events
//! that are not convered by the spec and not otherwise known by the application.
//! `CustomEvent`, `CustomRoomEvent`, and `CustomStateEvent` are simple implementations of their
//! respective event traits whose `content` field is simply a `serde_json::Value` value, which
//! represents arbitrary JSON.
//!
//! # Collections
//!
//! With the trait-based approach to events, it's easy to write generic collection types like
//! `Vec<Box<R: RoomEvent>>`.
//! However, there are APIs in the Matrix specification that involve heterogeneous collections of
//! events, i.e. a list of events of different event types.
//! Because Rust does not have a facility for arrays, vectors, or slices containing multiple
//! concrete types, ruma_events provides special collection types for this purpose.
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

#![feature(try_from)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(warnings)]

extern crate ruma_identifiers;
extern crate ruma_signatures;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::fmt::{Debug, Display, Error as FmtError, Formatter, Result as FmtResult};

use ruma_identifiers::{EventId, RoomId, UserId};
use serde::de::{Error as SerdeError, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

#[macro_use]
mod macros;

pub mod call;
/// Enums for heterogeneous collections of events.
pub mod collections {
    pub mod all;
    pub mod only;
}
pub mod direct;
pub mod presence;
pub mod receipt;
pub mod room;
pub mod stripped;
pub mod tag;
pub mod typing;

/// An error when attempting to convert a string to an enum that only accepts certain values.
#[derive(Clone, Copy, Debug)]
pub struct ParseError;

/// The type of an event.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum EventType {
    /// m.call.answer
    CallAnswer,
    /// m.call.candidates
    CallCandidates,
    /// m.call.hangup
    CallHangup,
    /// m.call.invite
    CallInvite,
    /// m.direct
    Direct,
    /// m.presence
    Presence,
    /// m.receipt
    Receipt,
    /// m.room.aliases
    RoomAliases,
    /// m.room.avatar
    RoomAvatar,
    /// m.room.canonical_alias
    RoomCanonicalAlias,
    /// m.room.create
    RoomCreate,
    /// m.room.guest_access
    RoomGuestAccess,
    /// m.room.history_visibility
    RoomHistoryVisibility,
    /// m.room.join_rules
    RoomJoinRules,
    /// m.room.member
    RoomMember,
    /// m.room.message
    RoomMessage,
    /// m.room.name
    RoomName,
    /// m.room.pinned_events
    RoomPinnedEvents,
    /// m.room.power_levels
    RoomPowerLevels,
    /// m.room.redaction
    RoomRedaction,
    /// m.room.third_party_invite
    RoomThirdPartyInvite,
    /// m.room.topic
    RoomTopic,
    /// m.tag
    Tag,
    /// m.typing
    Typing,
    /// Any event that is not part of the specification.
    Custom(String),
}

/// A basic event.
pub trait Event
where
    Self: Debug + for<'a> Deserialize<'a> + Serialize,
{
    /// The event-type-specific payload this event carries.
    type Content: Debug + for<'a> Deserialize<'a> + Serialize;

    /// The event's content.
    fn content(&self) -> &Self::Content;

    /// The type of the event.
    fn event_type(&self) -> &EventType;
}

/// An event within the context of a room.
pub trait RoomEvent: Event {
    /// The unique identifier for the event.
    fn event_id(&self) -> &EventId;

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    fn origin_server_ts(&self) -> u64;

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

event! {
    /// A custom basic event not covered by the Matrix specification.
    pub struct CustomEvent(Value) {}
}

room_event! {
    /// A custom room event not covered by the Matrix specification.
    pub struct CustomRoomEvent(Value) {}
}

state_event! {
    /// A custom state event not covered by the Matrix specification.
    pub struct CustomStateEvent(Value) {}
}

impl Display for EventType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        let event_type_str = match *self {
            EventType::CallAnswer => "m.call.answer",
            EventType::CallCandidates => "m.call.candidates",
            EventType::CallHangup => "m.call.hangup",
            EventType::CallInvite => "m.call.invite",
            EventType::Direct => "m.direct",
            EventType::Presence => "m.presence",
            EventType::Receipt => "m.receipt",
            EventType::RoomAliases => "m.room.aliases",
            EventType::RoomAvatar => "m.room.avatar",
            EventType::RoomCanonicalAlias => "m.room.canonical_alias",
            EventType::RoomCreate => "m.room.create",
            EventType::RoomGuestAccess => "m.room.guest_access",
            EventType::RoomHistoryVisibility => "m.room.history_visibility",
            EventType::RoomJoinRules => "m.room.join_rules",
            EventType::RoomMember => "m.room.member",
            EventType::RoomMessage => "m.room.message",
            EventType::RoomName => "m.room.name",
            EventType::RoomPinnedEvents => "m.room.pinned_events",
            EventType::RoomPowerLevels => "m.room.power_levels",
            EventType::RoomRedaction => "m.room.redaction",
            EventType::RoomThirdPartyInvite => "m.room.third_party_invite",
            EventType::RoomTopic => "m.room.topic",
            EventType::Tag => "m.tag",
            EventType::Typing => "m.typing",
            EventType::Custom(ref event_type) => event_type,
        };

        write!(f, "{}", event_type_str)
    }
}

impl<'a> From<&'a str> for EventType {
    fn from(s: &'a str) -> EventType {
        match s {
            "m.call.answer" => EventType::CallAnswer,
            "m.call.candidates" => EventType::CallCandidates,
            "m.call.hangup" => EventType::CallHangup,
            "m.call.invite" => EventType::CallInvite,
            "m.direct" => EventType::Direct,
            "m.presence" => EventType::Presence,
            "m.receipt" => EventType::Receipt,
            "m.room.aliases" => EventType::RoomAliases,
            "m.room.avatar" => EventType::RoomAvatar,
            "m.room.canonical_alias" => EventType::RoomCanonicalAlias,
            "m.room.create" => EventType::RoomCreate,
            "m.room.guest_access" => EventType::RoomGuestAccess,
            "m.room.history_visibility" => EventType::RoomHistoryVisibility,
            "m.room.join_rules" => EventType::RoomJoinRules,
            "m.room.member" => EventType::RoomMember,
            "m.room.message" => EventType::RoomMessage,
            "m.room.name" => EventType::RoomName,
            "m.room.pinned_events" => EventType::RoomPinnedEvents,
            "m.room.power_levels" => EventType::RoomPowerLevels,
            "m.room.redaction" => EventType::RoomRedaction,
            "m.room.third_party_invite" => EventType::RoomThirdPartyInvite,
            "m.room.topic" => EventType::RoomTopic,
            "m.tag" => EventType::Tag,
            "m.typing" => EventType::Typing,
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

            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
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

#[cfg(test)]
mod tests {
    use serde_json::{from_str, to_string};

    use super::EventType;

    #[test]
    fn event_types_serialize_to_display_form() {
        assert_eq!(
            to_string(&EventType::RoomCreate).unwrap(),
            r#""m.room.create""#
        );
    }

    #[test]
    fn custom_event_types_serialize_to_display_form() {
        assert_eq!(
            to_string(&EventType::Custom("io.ruma.test".to_string())).unwrap(),
            r#""io.ruma.test""#
        );
    }

    #[test]
    fn event_types_deserialize_from_display_form() {
        assert_eq!(
            from_str::<EventType>(r#""m.room.create""#).unwrap(),
            EventType::RoomCreate
        );
    }

    #[test]
    fn custom_event_types_deserialize_from_display_form() {
        assert_eq!(
            from_str::<EventType>(r#""io.ruma.test""#).unwrap(),
            EventType::Custom("io.ruma.test".to_string())
        )
    }
}
