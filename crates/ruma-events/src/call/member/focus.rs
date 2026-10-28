//! Types for MatrixRTC Focus/SFU configurations.

use ruma_common::OwnedServerName;
use ruma_macros::StringEnum;
use serde::{Deserialize, Serialize};

use crate::PrivOwnedStr;

/// Description of the SFU/Focus a membership can be connected to.
///
/// A focus can be any server powering the MatrixRTC session (SFU,
/// MCU). It serves as a node to redistribute RTC streams.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Focus {
    /// LiveKit is one possible type of SFU/Focus that can be used for a MatrixRTC session.
    Livekit(LivekitFocus),
}

/// The struct to describe LiveKit as a `preferred_foci`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct LivekitFocus {
    /// The alias where the LiveKit sessions can be reached.
    #[serde(rename = "livekit_alias", skip_serializing_if = "Option::is_none")]
    #[deprecated = "Newer versions of Element Call won't fill this field anymore."]
    pub alias: Option<String>,

    /// The WebSocket URL of the LiveKit SFU.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// The server name of the homeserver that grants access to the LiveKit SFU.
    ///
    /// This must be set when publishing on an SFU of a different homeserver than the sender's. If
    /// this is `None`, the SFU belongs to the sender's homeserver.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_name: Option<OwnedServerName>,

    /// The URL of the JWT service for the LiveKit instance.
    #[serde(rename = "livekit_service_url", skip_serializing_if = "Option::is_none")]
    #[deprecated = "Use `url` instead unless you require backwards compatibility with older versions of MSC4195."]
    pub service_url: Option<String>,
}

impl LivekitFocus {
    /// Initialize a [`LivekitFocus`] with the given SFU URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The WebSocket URL of the LiveKit SFU.
    #[allow(deprecated)]
    pub fn new(url: String) -> Self {
        Self { alias: None, url: Some(url), server_name: None, service_url: None }
    }

    /// Initialize a [`LivekitFocus`] with the given JWT service URL.
    ///
    /// # Arguments
    ///
    /// * `service_url` - The URL of the JWT service for the LiveKit instance.
    #[deprecated = "Use `LivekitFocus::new()` instead unless you require backwards compatibility with older versions of MSC4195."]
    #[allow(deprecated)]
    pub fn with_service_url(service_url: String) -> Self {
        Self { alias: None, url: None, server_name: None, service_url: Some(service_url) }
    }
}

/// Data to define the actively used Focus.
///
/// A focus can be any server powering the MatrixRTC session (SFU,
/// MCU). It serves as a node to redistribute RTC streams.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActiveFocus {
    /// LiveKit is one possible type of SFU/Focus that can be used for a MatrixRTC session.
    Livekit(ActiveLivekitFocus),
}

/// The fields to describe the `active_foci`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
#[derive(Clone, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum FocusSelection {
    /// Select the active focus by using the oldest membership and the oldest focus.
    OldestMembership,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
