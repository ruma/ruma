//! Modules for events in the `m.call` namespace.
//!
//! This module also contains types shared by events in its child namespaces.

pub mod answer;
pub mod candidates;
pub mod hangup;
pub mod invite;

use serde::{Deserialize, Serialize};

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
