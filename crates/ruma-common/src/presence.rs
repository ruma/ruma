//! Common types for the [presence module][presence].
//!
//! [presence]: https://matrix.org/docs/spec/client_server/r0.6.1#id62

use ruma_serde::StringEnum;

/// A description of a user's connectivity and availability for chat.
///
/// This type can hold an arbitrary string. To check for formats that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PresenceState {
    /// Disconnected from the service.
    Offline,

    /// Connected to the service.
    Online,

    /// Connected to the service but not available for chat.
    Unavailable,

    #[doc(hidden)]
    _Custom(String),
}

impl Default for PresenceState {
    fn default() -> Self {
        Self::Online
    }
}

impl Default for &'_ PresenceState {
    fn default() -> Self {
        &PresenceState::Online
    }
}

impl PresenceState {
    /// Creates a string slice from this `PresenceState`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}
