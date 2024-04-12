//! Common types for the [presence module][presence].
//!
//! [presence]: https://spec.matrix.org/latest/client-server-api/#presence

use serde::{Deserialize, Serialize};

/// A description of a user's connectivity and availability for chat.
#[derive(Clone, Default, PartialEq, Eq, Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum PresenceState {
    /// Disconnected from the service.
    Offline,

    /// Connected to the service.
    #[default]
    Online,

    /// Connected to the service but not available for chat.
    Unavailable,
}

impl Default for &'_ PresenceState {
    fn default() -> Self {
        &PresenceState::Online
    }
}
