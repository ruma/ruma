//! Modules for events in the `m.call` namespace.
//!
//! This module also contains types shared by events in its child namespaces.

pub mod answer;
pub mod candidates;
pub mod hangup;
pub mod invite;
#[cfg(feature = "unstable-msc3401")]
pub mod member;
pub mod negotiate;
#[cfg(feature = "unstable-msc4075")]
pub mod notify;
pub mod reject;
pub mod select_answer;

use serde::{Deserialize, Serialize};

/// A VoIP session description.
///
/// This is the same type as WebRTC's [`RTCSessionDescriptionInit`].
///
/// [`RTCSessionDescriptionInit`]: (https://www.w3.org/TR/webrtc/#dom-rtcsessiondescriptioninit):
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SessionDescription {
    /// The type of session description.
    ///
    /// This is the `type` field of `RTCSessionDescriptionInit`.
    #[serde(rename = "type")]
    pub session_type: String,

    /// The SDP text of the session description.
    ///
    /// Defaults to an empty string.
    #[serde(default)]
    pub sdp: String,
}

impl SessionDescription {
    /// Creates a new `SessionDescription` with the given session type and SDP text.
    pub fn new(session_type: String, sdp: String) -> Self {
        Self { session_type, sdp }
    }
}

/// The capabilities of a client in a VoIP call.
#[cfg(feature = "unstable-msc2747")]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct CallCapabilities {
    /// Whether this client supports [DTMF].
    ///
    /// Defaults to `false`.
    ///
    /// [DTMF]: https://w3c.github.io/webrtc-pc/#peer-to-peer-dtmf
    #[serde(rename = "m.call.dtmf", default)]
    pub dtmf: bool,
}

#[cfg(feature = "unstable-msc2747")]
impl CallCapabilities {
    /// Creates a default `CallCapabilities`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether this `CallCapabilities` only contains default values.
    pub fn is_default(&self) -> bool {
        !self.dtmf
    }
}
