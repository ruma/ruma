use std::fmt::Debug;

use serde::{Deserialize, Serialize};

/// The type of an event.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum EventType {
    /// m.direct
    Direct,

    /// m.room.aliases
    RoomAliases,

    /// m.room.redaction
    RoomRedaction,
}

/// A basic event.
pub trait Event
where
    Self: Debug + Serialize,
{
    /// The type of the event.
    const EVENT_TYPE: EventType;

    /// The type of this event's `content` field.
    type Content: Debug + Serialize;

    /// The event's content.
    fn content(&self) -> &Self::Content;

    /// The type of the event.
    fn event_type(&self) -> EventType {
        Self::EVENT_TYPE
    }
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

pub struct InvalidEvent;

impl From<serde_json::Error> for InvalidEvent {
    fn from(_: serde_json::Error) -> Self {
        Self
    }
}

// See note about wrapping macro expansion in a module from `src/lib.rs`
pub mod common_case {
    use super::Event;

    use ruma_events_macros::ruma_event;

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
}

pub mod extra_fields {
    use super::Event;

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
    use super::Event;

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
