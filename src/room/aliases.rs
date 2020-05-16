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

impl EventContent for AliasesEventContent {
    fn event_type(&self) -> &str {
        "m.room.aliases"
    }

    fn from_parts(event_type: &str, content: Box<RawJsonValue>) -> Result<Self, InvalidEvent> {
        if event_type != "m.room.aliases" {
            return Err(InvalidEvent {
                kind: InvalidEventKind::Deserialization,
                message: format!("expected `m.room.aliases` found {}", event_type),
            });
        }

        let ev_json = EventJson::from(content);
        ev_json.deserialize()
    }
}

impl RoomEventContent for AliasesEventContent {}

impl StateEventContent for AliasesEventContent {}
