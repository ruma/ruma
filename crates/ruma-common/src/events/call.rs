//! Modules for events in the `m.call` namespace.
//!
//! This module also contains types shared by events in its child namespaces.

pub mod answer;
pub mod candidates;
pub mod hangup;
pub mod invite;
#[cfg(feature = "unstable-msc2746")]
pub mod negotiate;
#[cfg(feature = "unstable-msc2746")]
pub mod reject;
#[cfg(feature = "unstable-msc2746")]
pub mod select_answer;
#[cfg(feature = "unstable-msc2746")]
mod version_serde;

use js_int::UInt;
use serde::{Deserialize, Serialize};

use crate::{serde::StringEnum, PrivOwnedStr};

/// A VoIP session description.
///
/// With the `unstable-msc2746` feature, this is the same type as WebRTC's
/// [`RTCSessionDescriptionInit`].
///
/// [`RTCSessionDescriptionInit`]: (https://www.w3.org/TR/webrtc/#dom-rtcsessiondescriptioninit):
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SessionDescription {
    /// The type of session description.
    #[serde(rename = "type")]
    pub session_type: SessionDescriptionType,

    /// The SDP text of the session description.
    ///
    /// With the `unstable-msc2746` feature, this field is unused if the type is `rollback` and
    /// defaults to an empty string.
    #[cfg_attr(feature = "unstable-msc2746", serde(default))]
    pub sdp: String,
}

impl SessionDescription {
    /// Creates a new `SessionDescription` with the given session type and SDP text.
    pub fn new(session_type: SessionDescriptionType, sdp: String) -> Self {
        Self { session_type, sdp }
    }
}

/// The type of VoIP session description.
///
/// With the `unstable-msc2746` feature, this is the same type as WebRTC's [`RTCSdpType`].
///
/// [`RTCSdpType`]: (https://www.w3.org/TR/webrtc/#dom-rtcsdptype):
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "lowercase")]
#[non_exhaustive]
pub enum SessionDescriptionType {
    /// An answer.
    Answer,

    /// An offer.
    Offer,

    /// An answer that is not final.
    #[cfg(feature = "unstable-msc2746")]
    PrAnswer,

    /// Cancel the current negotiation and rollback to the stable state.
    #[cfg(feature = "unstable-msc2746")]
    Rollback,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// A VoIP answer session description.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "answer")]
pub struct AnswerSessionDescription {
    /// The SDP text of the session description.
    pub sdp: String,
}

impl AnswerSessionDescription {
    /// Creates a new `AnswerSessionDescription` with the given SDP text.
    pub fn new(sdp: String) -> Self {
        Self { sdp }
    }
}

/// A VoIP offer session description.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename = "offer")]
pub struct OfferSessionDescription {
    /// The SDP text of the session description.
    pub sdp: String,
}

impl OfferSessionDescription {
    /// Creates a new `OfferSessionDescription` with the given SDP text.
    pub fn new(sdp: String) -> Self {
        Self { sdp }
    }
}

/// The version of a VoIP call.
#[cfg(feature = "unstable-msc2746")]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum CallVersion {
    /// A stable version.
    ///
    /// According to the fields and event types used, this can be `0` or `1`.
    Stable(UInt),

    /// A namespaced version.
    Namespaced(String),
}

#[cfg(feature = "unstable-msc2746")]
impl From<UInt> for CallVersion {
    fn from(u: UInt) -> Self {
        Self::Stable(u)
    }
}

#[cfg(feature = "unstable-msc2746")]
impl From<String> for CallVersion {
    fn from(s: String) -> Self {
        Self::Namespaced(s)
    }
}

#[cfg(feature = "unstable-msc2746")]
impl From<&str> for CallVersion {
    fn from(s: &str) -> Self {
        Self::from(s.to_owned())
    }
}

/// The capabilities of a client in a VoIP call.
#[cfg(feature = "unstable-msc2746")]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct CallCapabilities {
    /// Whether this client supports [DTMF].
    ///
    /// [DTMF]: https://w3c.github.io/webrtc-pc/#peer-to-peer-dtmf
    #[serde(rename = "m.call.dtmf", default)]
    pub dtmf: bool,
}

#[cfg(feature = "unstable-msc2746")]
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
