//! Types for MatrixRTC Focus/SFU configurations.

use ruma_macros::StringEnum;
use serde::{Deserialize, Serialize};

use crate::PrivOwnedStr;

/// Description of the SFU/Focus a membership can be connected to.
///
/// A focus can be any server powering the MatrixRTC session (SFU,
/// MCU). It serves as a node to redistribute RTC streams.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Focus {
    /// LiveKit is one possible type of SFU/Focus that can be used for a MatrixRTC session.
    Livekit(LivekitFocus),
}

/// The struct to describe LiveKit as a `preferred_foci`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct LivekitFocus {
    /// The alias where the LiveKit sessions can be reached.
    #[serde(rename = "livekit_alias")]
    pub alias: String,

    /// The URL of the JWT service for the LiveKit instance.
    #[serde(rename = "livekit_service_url")]
    pub service_url: String,
}

impl LivekitFocus {
    /// Initialize a [`LivekitFocus`].
    ///
    /// # Arguments
    ///
    /// * `alias` - The alias with which the LiveKit sessions can be reached.
    /// * `service_url` - The url of the JWT server for the LiveKit instance.
    pub fn new(alias: String, service_url: String) -> Self {
        Self { alias, service_url }
    }
}

/// Data to define the actively used Focus.
///
/// A focus can be any server powering the MatrixRTC session (SFU,
/// MCU). It serves as a node to redistribute RTC streams.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActiveFocus {
    /// LiveKit is one possible type of SFU/Focus that can be used for a MatrixRTC session.
    Livekit(ActiveLivekitFocus),
}

/// The fields to describe the `active_foci`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ActiveLivekitFocus {
    /// The selection method used to select the LiveKit focus for the rtc session.
    pub focus_selection: FocusSelection,
}

impl ActiveLivekitFocus {
    /// Initialize a [`ActiveLivekitFocus`].
    ///
    /// # Arguments
    ///
    /// * `focus_selection` - The selection method used to select the LiveKit focus for the rtc
    ///   session.
    pub fn new() -> Self {
        Self { focus_selection: FocusSelection::OldestMembership }
    }
}

/// How to select the active focus for LiveKit
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum FocusSelection {
    /// Select the active focus by using the oldest membership and the oldest focus.
    OldestMembership,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
