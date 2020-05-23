//! Types for the *m.room.aliases* event.

use ruma_events_macros::{FromRaw, StateEventContent};
use ruma_identifiers::RoomAliasId;
use serde::Serialize;
use serde_json::value::RawValue as RawJsonValue;

use crate::{
    error::{InvalidEvent, InvalidEventKind},
    EventContent, EventJson, RoomEventContent, StateEventContent,
};

/// Informs the room about what room aliases it has been given.
#[derive(Clone, Debug, Serialize, FromRaw, StateEventContent)]
#[ruma_event(type = "m.room.aliases")]
pub struct AliasesEventContent {
    /// A list of room aliases.
    pub aliases: Vec<RoomAliasId>,
}
