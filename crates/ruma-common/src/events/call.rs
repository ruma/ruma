//! Modules for events in the `m.call` namespace.
//!
//! This module also contains types shared by events in its child namespaces.

pub mod answer;
pub mod candidates;
pub mod hangup;
pub mod invite;

use serde::{Deserialize, Serialize};

use crate::{serde::StringEnum, PrivOwnedStr};

/// A VoIP answer session description.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct AnswerSessionDescription {
    /// The type of session description.
    #[serde(rename = "type")]
    pub session_type: AnswerSessionDescriptionType,

    /// The SDP text of the session description.
    pub sdp: String,
}

impl AnswerSessionDescription {
    /// Creates a new `AnswerSessionDescription` with the given SDP text.
    pub fn new(sdp: String) -> Self {
        Self { session_type: AnswerSessionDescriptionType::Answer, sdp }
    }
}

/// The type of VoIP answer session description.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AnswerSessionDescriptionType {
    /// An answer.
    Answer,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// A VoIP offer session description.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct OfferSessionDescription {
    /// The type of session description.
    #[serde(rename = "type")]
    pub session_type: OfferSessionDescriptionType,

    /// The SDP text of the session description.
    pub sdp: String,
}

impl OfferSessionDescription {
    /// Creates a new `OfferSessionDescription` with the given SDP text.
    pub fn new(sdp: String) -> Self {
        Self { session_type: OfferSessionDescriptionType::Offer, sdp }
    }
}

/// The type of VoIP offer session description.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum OfferSessionDescriptionType {
    /// An offer.
    Offer,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
