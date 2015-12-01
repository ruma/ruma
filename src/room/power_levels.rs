//! Types for the *m.room.power_levels* event.

use std::collections::HashMap;

use core::{Event, EventType, RoomEvent, StateEvent};

/// Defines the power levels (privileges) of users in the room.
pub struct PowerLevelsEvent<'a, 'b> {
    content: PowerLevelsEventContent<'a>,
    event_id: &'a str,
    prev_content: Option<PowerLevelsEventContent<'b>>,
    room_id: &'a str,
    user_id: &'a str,
}

impl<'a, 'b> Event<'a, PowerLevelsEventContent<'a>> for PowerLevelsEvent<'a, 'b> {
    fn content(&'a self) -> &'a PowerLevelsEventContent<'a> {
        &self.content
    }

    fn event_type(&self) -> EventType {
        EventType::RoomPowerLevels
    }
}

impl<'a, 'b> RoomEvent<'a, PowerLevelsEventContent<'a>> for PowerLevelsEvent<'a, 'b> {
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

impl<'a, 'b> StateEvent<'a, 'b, PowerLevelsEventContent<'a>> for PowerLevelsEvent<'a, 'b> {
    fn prev_content(&'a self) -> Option<&'b PowerLevelsEventContent> {
        match self.prev_content {
            Some(ref prev_content) => Some(prev_content),
            None => None,
        }
    }
}

/// The payload of a `PowerLevelsEvent`.
pub struct PowerLevelsEventContent<'a> {
    ban: u64,
    events: &'a HashMap<&'a str, u64>,
    events_default: u64,
    kick: u64,
    redact: u64,
    state_default: u64,
    users: &'a HashMap<&'a str, u64>,
    users_default: u64,
}
