//! Types for the *m.room.history_visibility* event.

use ruma_events_macros::{FromRaw, StateEventContent};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// This event controls whether a member of a room can see the events that happened in a room
/// from before they joined.
#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[ruma_event(type = "m.room.history_visibility")]
pub struct HistoryVisibilityEventContent {
    /// Who can see the room history.
    pub history_visibility: HistoryVisibility,
}

/// Who can see a room's history.
#[derive(Clone, Copy, Debug, PartialEq, Display, EnumString, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum HistoryVisibility {
    /// Previous events are accessible to newly joined members from the point they were invited
    /// onwards. Events stop being accessible when the member's state changes to something other
    /// than *invite* or *join*.
    Invited,

    /// Previous events are accessible to newly joined members from the point they joined the room
    /// onwards. Events stop being accessible when the member's state changes to something other
    /// than *join*.
    Joined,

    /// Previous events are always accessible to newly joined members. All events in the room are
    /// accessible, even those sent when the member was not a part of the room.
    Shared,

    /// All events while this is the `HistoryVisibility` value may be shared by any
    /// participating homeserver with anyone, regardless of whether they have ever joined the room.
    WorldReadable,
}
