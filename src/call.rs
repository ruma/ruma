//! Modules for events in the *m.call* namespace.
//!
//! This module also contains types shared by events in its child namespaces.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

pub mod answer;
pub mod candidates;
pub mod hangup;
pub mod invite;

/// A VoIP session description.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SessionDescription {
    /// The type of session description.
    #[serde(rename = "type")]
    pub session_type: SessionDescriptionType,

    /// The SDP text of the session description.
    pub sdp: String,
}

/// The type of VoIP session description.
#[derive(Clone, Copy, Debug, Display, EnumString, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum SessionDescriptionType {
    /// An answer.
    Answer,

    /// An offer.
    Offer,
}
