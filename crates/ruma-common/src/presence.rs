//! Common types for the [presence module][presence].
//!
//! [presence]: https://spec.matrix.org/v1.19/client-server-api/#presence

#[cfg(feature = "unstable-msc4532")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "unstable-msc4532")]
use crate::serde::JsonObject;
use crate::{PrivOwnedStr, serde::StringEnum};

/// A description of a user's connectivity and availability for chat.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Default, StringEnum)]
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

    /// The user is available to reply.
    ///
    /// This uses the unstable prefix defined in [MSC4532].
    ///
    /// [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532
    #[cfg(feature = "unstable-msc4532")]
    Active,

    /// The user has a connected client and may reply (potentially unreachable).
    ///
    /// This uses the unstable prefix defined in [MSC4532].
    ///
    /// [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532
    #[cfg(feature = "unstable-msc4532")]
    Idle,

    /// The user is unavailable to reply (fully unreachable).
    ///
    /// This uses the unstable prefix defined in [MSC4532].
    ///
    /// [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532
    #[cfg(feature = "unstable-msc4532")]
    Busy,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl PresenceState {
    /// If this [`PresenceState`] is a legacy (non-[MSC4532]) state, return the equivalent MSC4532
    /// state.
    ///
    /// This uses the behavior map as defined in the proposal.
    ///
    /// [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532
    #[cfg(feature = "unstable-msc4532")]
    pub fn as_msc4532_state(&self, currently_active: bool) -> Self {
        match (self, currently_active) {
            (Self::Online, true) => Self::Active,
            (Self::Online, false) => Self::Idle,
            (Self::Unavailable, _) => Self::Idle,
            (state, _) => state.clone(),
        }
    }

    /// If this [`PresenceState`] is an MSC4532 state, return the equivalent legacy (non-MSC4532)
    /// state.
    ///
    /// This uses the behavior map as defined in the proposal.
    ///
    /// [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532
    #[cfg(feature = "unstable-msc4532")]
    pub fn as_legacy_state(&self) -> Self {
        match self {
            Self::Active => Self::Online,
            Self::Idle => Self::Unavailable,
            Self::Busy => Self::Unavailable,
            state => state.clone(),
        }
    }

    /// Return a reasonable `currently_active` value for this [`PresenceState`].
    #[cfg(feature = "unstable-msc4532")]
    pub fn currently_active(&self) -> bool {
        matches!(self, Self::Active | Self::Online)
    }
}

impl Default for &'_ PresenceState {
    fn default() -> Self {
        #[cfg(feature = "unstable-msc4532")]
        {
            &PresenceState::Active
        }
        #[cfg(not(feature = "unstable-msc4532"))]
        {
            &PresenceState::Online
        }
    }
}

/// A user's extensible status.
///
/// This uses the unstable prefix defined in [MSC4532].
///
/// [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532
#[cfg(feature = "unstable-msc4532")]
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct PresenceStatus {
    /// An optional description to accompany the presence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,

    /// Remaining content.
    #[serde(flatten)]
    pub data: JsonObject,
}

#[cfg(feature = "unstable-msc4532")]
impl PresenceStatus {
    /// Creates a new `PresenceStatus` with the given message.
    pub fn new(msg: Option<String>) -> Self {
        Self { msg, data: JsonObject::new() }
    }
}
