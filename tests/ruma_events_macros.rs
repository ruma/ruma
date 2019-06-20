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
