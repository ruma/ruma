//! Types for the *m.room.power_levels* event.

use std::collections::HashMap;

use events::StateEvent;

/// Defines the power levels (privileges) of users in the room.
pub type PowerLevelsEvent = StateEvent<PowerLevelsEventContent>;

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
