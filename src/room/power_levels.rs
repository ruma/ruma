//! Types for the *m.room.power_levels* event.

use std::collections::HashMap;

use core::EventType;

/// Defines the power levels (privileges) of users in the room.
pub struct PowerLevelsEvent {
    content: PowerLevelsEventContent,
    event_id: String,
    event_type: EventType,
    prev_content: Option<PowerLevelsEventContent>,
    room_id: String,
    state_key: String,
    user_id: String,
}

/// The payload of a `PowerLevelsEvent`.
pub struct PowerLevelsEventContent {
    ban: u64,
    events: HashMap<String, u64>,
    events_default: u64,
    kick: u64,
    redact: u64,
    state_default: u64,
    users: HashMap<String, u64>,
    users_default: u64,
}
