use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// A description of a user's connectivity and availability for chat.
#[derive(Clone, Copy, Debug, PartialEq, Display, EnumString, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum PresenceState {
    /// Disconnected from the service.
    Offline,

    /// Connected to the service.
    Online,

    /// Connected to the service but not available for chat.
    Unavailable,
}

impl Default for PresenceState {
    fn default() -> Self {
        Self::Online
    }
}
