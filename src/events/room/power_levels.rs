//! Types for the *m.room.power_levels* event.

use std::collections::HashMap;

use events::EventType;

/// Defines the power levels (privileges) of users in the room.
#[derive(Debug, Deserialize, Serialize)]
pub struct PowerLevelsEvent {
    pub content: PowerLevelsEventContent,
    pub event_id: String,
    pub event_type: EventType,
    pub prev_content: Option<PowerLevelsEventContent>,
    pub room_id: String,
    pub state_key: String,
    pub user_id: String,
}

/// The payload of a `PowerLevelsEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct PowerLevelsEventContent {
    pub ban: u64,
    pub events: HashMap<String, u64>,
    pub events_default: u64,
    pub kick: u64,
    pub redact: u64,
    pub state_default: u64,
    pub users: HashMap<String, u64>,
    pub users_default: u64,
}
