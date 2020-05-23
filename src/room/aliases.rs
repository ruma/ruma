//! Types for the *m.room.aliases* event.

use ruma_events_macros::ruma_event;
use ruma_identifiers::RoomAliasId;
use serde_json::value::RawValue as RawJsonValue;

use crate::{
    error::{InvalidEvent, InvalidEventKind},
    EventContent, EventJson, RoomEventContent, StateEventContent,
};

ruma_event! {
    /// Informs the room about what room aliases it has been given.
    AliasesEvent {
        kind: StateEvent,
        event_type: "m.room.aliases",
        content: {
            /// A list of room aliases.
            pub aliases: Vec<RoomAliasId>,
        },
    }
}
