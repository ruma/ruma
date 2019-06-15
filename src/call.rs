//! Modules for events in the *m.call* namespace.
//!
//! This module also contains types shared by events in its child namespaces.

use serde::{Deserialize, Serialize};

pub mod answer;
pub mod candidates;
pub mod hangup;
pub mod invite;

/// A VoIP session description.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SessionDescription {
    /// The type of session description.
    #[serde(rename = "type")]
    pub session_type: SessionDescriptionType,

    /// The SDP text of the session description.
    pub sdp: String,
}

/// The type of VoIP session description.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum SessionDescriptionType {
    /// An answer.
    #[serde(rename = "answer")]
    Answer,

    /// An offer.
    #[serde(rename = "offer")]
    Offer,

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to `ruma-events`.
    #[doc(hidden)]
    #[serde(skip)]
    __Nonexhaustive,
}

impl_enum! {
    SessionDescriptionType {
        Answer => "answer",
        Offer => "offer",
    }
}
