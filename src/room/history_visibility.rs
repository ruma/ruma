//! Types for the *m.room.history_visibility* event.

use core::EventType;

/// This event controls whether a member of a room can see the events that happened in a room from
/// before they joined.
pub struct HistoryVisibilityEvent {
    content: HistoryVisibilityEventContent,
    event_id: String,
    event_type: EventType,
    prev_content: Option<HistoryVisibilityEventContent>,
    room_id: String,
    state_key: String,
    user_id: String,
}

/// The payload of a `HistoryVisibilityEvent`.
pub struct HistoryVisibilityEventContent {
    /// Who can see the room history.
    history_visibility: HistoryVisibility,
}

/// Who can see a room's history.
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
