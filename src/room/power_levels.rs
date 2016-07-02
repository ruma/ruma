//! Types for the *m.room.power_levels* event.

use std::collections::HashMap;

use StateEvent;

/// Defines the power levels (privileges) of users in the room.
pub type PowerLevelsEvent = StateEvent<PowerLevelsEventContent, ()>;

/// The payload of a `PowerLevelsEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct PowerLevelsEventContent {
    /// The level required to ban a user.
    pub ban: u64,

    /// The level required to send specific event types.
    ///
    /// This is a mapping from event type to power level required.
    pub events: HashMap<String, u64>,

    /// The default level required to send message events.
    pub events_default: u64,

    /// The level required to invite a user.
    pub invite: u64,

    /// The level required to kick a user.
    pub kick: u64,

    /// The level required to redact an event.
    pub redact: u64,

    /// The default level required to send state events.
    pub state_default: u64,

    /// The power levels for specific users.
    ///
    /// This is a mapping from `user_id` to power level for that user.
    pub users: HashMap<String, u64>,

    /// The default power level for every user in the room.
    pub users_default: u64,
}
