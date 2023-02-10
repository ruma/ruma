//! Common types for the [presence module][presence].
//!
//! [presence]: https://spec.matrix.org/latest/client-server-api/#presence

use crate::{serde::StringEnum, PrivOwnedStr};

/// A description of a user's connectivity and availability for chat.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Default, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PresenceState {
    /// Disconnected from the service.
    Offline,

    /// Connected to the service.
    #[default]
    Online,

    /// Connected to the service but not available for chat.
    Unavailable,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl Default for &'_ PresenceState {
    fn default() -> Self {
        &PresenceState::Online
    }
}
