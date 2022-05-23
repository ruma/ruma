//! Common types for the [`m.room.power_levels` event][power_levels].
//!
//! [power_levels]: https://spec.matrix.org/v1.2/client-server-api/#mroompower_levels

use js_int::{int, Int};
use serde::{Deserialize, Serialize};

/// The power level requirements for specific notification types.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct NotificationPowerLevels {
    /// The level required to trigger an `@room` notification.
    ///
    /// If you activate the `compat` feature, deserialization will work for stringified
    /// integers too.
    #[cfg_attr(
        feature = "compat",
        serde(deserialize_with = "crate::serde::deserialize_v1_powerlevel")
    )]
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

    pub(crate) fn is_default(&self) -> bool {
        self.room == default_power_level()
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
