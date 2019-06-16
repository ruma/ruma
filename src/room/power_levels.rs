//! Types for the *m.room.power_levels* event.

use std::collections::HashMap;

use js_int::{Int, UInt};
use ruma_identifiers::UserId;
use serde::{Deserialize, Serialize};

use crate::EventType;

state_event! {
    /// Defines the power levels (privileges) of users in the room.
    pub struct PowerLevelsEvent(PowerLevelsEventContent) {}
}

/// The payload of a `PowerLevelsEvent`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PowerLevelsEventContent {
    /// The level required to ban a user.
    #[serde(default = "default_power_level")]
    pub ban: Int,

    /// The level required to send specific event types.
    ///
    /// This is a mapping from event type to power level required.
    pub events: HashMap<EventType, Int>,

    /// The default level required to send message events.
    #[serde(default)]
    pub events_default: Int,

    /// The level required to invite a user.
    #[serde(default = "default_power_level")]
    pub invite: Int,

    /// The level required to kick a user.
    #[serde(default = "default_power_level")]
    pub kick: Int,

    /// The level required to redact an event.
    #[serde(default = "default_power_level")]
    pub redact: Int,

    /// The default level required to send state events.
    #[serde(default = "default_power_level")]
    pub state_default: Int,

    /// The power levels for specific users.
    ///
    /// This is a mapping from `user_id` to power level for that user.
    pub users: HashMap<UserId, Int>,

    /// The default power level for every user in the room.
    #[serde(default)]
    pub users_default: Int,

    /// The power level requirements for specific notification types.
    ///
    /// This is a mapping from `key` to power level for that notifications key.
    pub notifications: NotificationPowerLevels,
}

/// The power level requirements for specific notification types.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct NotificationPowerLevels {
    /// The level required to trigger an `@room` notification.
    #[serde(default = "default_power_level")]
    pub room: Int,
}

/// Used to default power levels to 50 during deserialization.
fn default_power_level() -> Int {
    Int::from(50)
}
