//! Common types for the [`m.room.power_levels` event][power_levels].
//!
//! [power_levels]: https://spec.matrix.org/latest/client-server-api/#mroompower_levels

use js_int::{int, Int};
use ruma_macros::StringEnum;
use serde::{Deserialize, Serialize};

use crate::PrivOwnedStr;

/// The power level requirements for specific notification types.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct NotificationPowerLevels {
    /// The level required to trigger an `@room` notification.
    ///
    /// Defaults to `50`.
    #[serde(
        default = "default_power_level",
        deserialize_with = "crate::serde::deserialize_v1_powerlevel"
    )]
    pub room: Int,
}

impl NotificationPowerLevels {
    /// Create a new `NotificationPowerLevels` with all-default values.
    pub fn new() -> Self {
        Self { room: default_power_level() }
    }

    /// Value associated with the given `key`.
    pub fn get(&self, key: &NotificationPowerLevelsKey) -> Option<&Int> {
        match key {
            NotificationPowerLevelsKey::Room => Some(&self.room),
            NotificationPowerLevelsKey::_Custom(_) => None,
        }
    }

    /// Whether all fields have their default values.
    pub fn is_default(&self) -> bool {
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

/// The possible keys of [`NotificationPowerLevels`].
#[derive(Clone, StringEnum)]
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[ruma_enum(rename_all = "lowercase")]
#[non_exhaustive]
pub enum NotificationPowerLevelsKey {
    /// The key for `@room` notifications.
    Room,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
