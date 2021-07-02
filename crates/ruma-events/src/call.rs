//! Modules for events in the *m.call* namespace.
//!
//! This module also contains types shared by events in its child namespaces.

use ruma_serde::StringEnum;
use serde::{Deserialize, Serialize};

pub mod answer;
pub mod candidates;
pub mod hangup;
pub mod invite;

/// A VoIP session description.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SessionDescription {
    /// The type of session description.
    #[serde(rename = "type")]
    pub session_type: SessionDescriptionType,

    /// The SDP text of the session description.
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
/// This type can hold an arbitrary string. To check for formats that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SessionDescriptionType {
    /// An answer.
    Answer,

    /// An offer.
    Offer,

    #[doc(hidden)]
    _Custom(String),
}

impl SessionDescriptionType {
    /// Creates a string slice from this `SessionDescriptionType`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}
