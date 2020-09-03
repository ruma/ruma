//! Common types for the [presence module][presence]
//!
//! [presence]: https://matrix.org/docs/spec/client_server/r0.6.1#id62

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// A description of a user's connectivity and availability for chat.
#[derive(Clone, Copy, Debug, PartialEq, Display, EnumString, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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
