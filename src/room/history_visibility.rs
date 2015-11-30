//! Types for the *m.room.history_visibility* event.

use core::{Event, EventType, RoomEvent, StateEvent};

/// This event controls whether a member of a room can see the events that happened in a room from
/// before they joined.
pub struct HistoryVisibilityEvent<'a, 'b> {
    content: HistoryVisibilityEventContent<'a>,
    event_id: &'a str,
    prev_content: Option<HistoryVisibilityEventContent<'b>>,
    room_id: &'a str,
    user_id: &'a str,
}

impl<'a, 'b> Event<'a, HistoryVisibilityEventContent<'a>> for HistoryVisibilityEvent<'a, 'b> {
    fn content(&'a self) -> &'a HistoryVisibilityEventContent<'a> {
        &self.content
    }

    fn event_type(&self) -> EventType {
        EventType::RoomHistoryVisibility
    }
}

impl<'a, 'b> RoomEvent<'a, HistoryVisibilityEventContent<'a>> for HistoryVisibilityEvent<'a, 'b> {
    fn event_id(&'a self) -> &'a str {
        &self.event_id
    }

    fn room_id(&'a self) -> &'a str {
        &self.room_id
    }

    fn user_id(&'a self) -> &'a str {
        &self.user_id
    }
}

impl<'a, 'b> StateEvent<'a, 'b, HistoryVisibilityEventContent<'a>>
    for HistoryVisibilityEvent<'a, 'b>
{
    fn prev_content(&'a self) -> Option<&'b HistoryVisibilityEventContent> {
        match self.prev_content {
            Some(ref prev_content) => Some(prev_content),
            None => None,
        }
    }
}

/// The payload of a `HistoryVisibilityEvent`.
pub struct HistoryVisibilityEventContent<'a> {
    /// Who can see the room history.
    history_visibility: &'a HistoryVisibility,
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

