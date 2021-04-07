//! Common types for the [`m.room.power_levels` event][power_levels].
//!
//! [power_levels]: https://matrix.org/docs/spec/client_server/r0.6.1#m-room-power-levels

use js_int::{int, Int};
use serde::{Deserialize, Serialize};

/// The power level requirements for specific notification types.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct NotificationPowerLevels {
    /// The level required to trigger an `@room` notification.
    ///
    /// If you activate the `compat` feature, this field being a string in JSON will give
    /// you an `Int` here.
    #[cfg_attr(feature = "compat", serde(deserialize_with = "ruma_serde::int_or_string_to_int"))]
    #[serde(default = "default_power_level")]
    pub room: Int,
}

impl NotificationPowerLevels {
    /// Create a new `NotificationPowerLevels` with all-default values.
    pub fn new() -> Self {
        Self { room: default_power_level() }
    }

    /// Value associated with the given `key`.
    pub fn get(&self, key: &str) -> Option<&Int> {
        match key {
            "room" => Some(&self.room),
            _ => None,
        }
    }
}

impl Default for NotificationPowerLevels {
    fn default() -> Self {
        Self::new()
    }
}

/// Used to default power levels to 50 during deserialization.
pub fn default_power_level() -> Int {
    int!(50)
}
