//! Types for matrixRTC Focus/SFU configurations.

use serde::{Deserialize, Serialize};

/// Description of the SFU/Focus a membership can be connected to.
///
/// A focus can be any server powering the matrixRTC session (SFU,
/// MCU). It serves as a node to redistribute RTC streams.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Focus {
    /// Livekit is one possible type of SFU/Focus that can be used for a matrixRTC session.
    Livekit(LivekitFocus),
}

/// The fields to describe livekit as an `preferred_foci`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct LivekitFocus {
    /// The alias where the livekit sessions can be reached.
    #[serde(rename = "livekit_alias")]
    pub alias: String,

    /// The url of the jwt server for the livekit instance.
    #[serde(rename = "livekit_service_url")]
    pub service_url: String,
}

impl LivekitFocus {
    /// Initialize a [`LivekitFocus`].
    ///
    /// # Arguments
    ///
    /// * `alias` - The alias where the livekit sessions can be reached.
    /// * `service_url` - The url of the jwt server for the livekit instance.
    pub fn new(alias: String, service_url: String) -> Self {
        Self { alias, service_url }
    }
}

/// Data to define the actively used Focus.
///
/// A focus can be any server powering the matrixRTC session (SFU,
/// MCU). It serves as a node to redistribute RTC streams.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActiveFocus {
    /// Livekit is one possible type of SFU/Focus that can be used for a matrixRTC session.
    Livekit(ActiveLivekitFocus),
}

/// The fields to describe the `active_foci`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ActiveLivekitFocus {
    /// The url of the jwt server for the livekit instance.
    pub focus_select: FocusSelection,
}

impl ActiveLivekitFocus {
    /// Initialize a [`LivekitFocus`].
    ///
    /// # Arguments
    ///
    /// * `alias` - The alias where the livekit sessions can be reached.
    /// * `service_url` - The url of the jwt server for the livekit instance.
    pub fn new() -> Self {
        Self { focus_select: FocusSelection::OldestMembership }
    }
}
/// How to select the active focus for livekit
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum FocusSelection {
    /// Select the active focus by using the oldest membership and the oldest focus.
    #[serde(rename = "oldest_membership")]
    OldestMembership,
}
