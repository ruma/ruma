//! Types for the *m.room.history_visibility* event.

use serde::{Deserialize, Serialize};

state_event! {
    /// This event controls whether a member of a room can see the events that happened in a room
    /// from before they joined.
    pub struct HistoryVisibilityEvent(HistoryVisibilityEventContent) {}
}

/// The payload of a `HistoryVisibilityEvent`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct HistoryVisibilityEventContent {
    /// Who can see the room history.
    pub history_visibility: HistoryVisibility,
}

/// Who can see a room's history.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum HistoryVisibility {
    /// Previous events are accessible to newly joined members from the point they were invited
    /// onwards. Events stop being accessible when the member's state changes to something other
    /// than *invite* or *join*.
    #[serde(rename = "invited")]
    Invited,

    /// Previous events are accessible to newly joined members from the point they joined the room
    /// onwards. Events stop being accessible when the member's state changes to something other
    /// than *join*.
    #[serde(rename = "joined")]
    Joined,

    /// Previous events are always accessible to newly joined members. All events in the room are
    /// accessible, even those sent when the member was not a part of the room.
    #[serde(rename = "shared")]
    Shared,

    /// All events while this is the `HistoryVisibility` value may be shared by any
    /// participating homeserver with anyone, regardless of whether they have ever joined the room.
    #[serde(rename = "world_readable")]
    WorldReadable,

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to `ruma-events`.
    #[doc(hidden)]
    #[serde(skip)]
    __Nonexhaustive,
}

impl_enum! {
    HistoryVisibility {
        Invited => "invited",
        Joined => "joined",
        Shared => "shared",
        WorldReadable => "world_readable",
    }
}
