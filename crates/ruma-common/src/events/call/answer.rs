//! Types for the [`m.call.answer`] event.
//!
//! [`m.call.answer`]: https://spec.matrix.org/v1.2/client-server-api/#mcallanswer

#[cfg(feature = "unstable-msc2746")]
use js_int::uint;
#[cfg(not(feature = "unstable-msc2746"))]
use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::AnswerSessionDescription;
#[cfg(feature = "unstable-msc2746")]
use super::{CallCapabilities, CallVersion};
#[cfg(feature = "unstable-msc2746")]
use crate::OwnedVoipId;

/// The content of an `m.call.answer` event.
///
/// This event is sent by the callee when they wish to answer the call.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.call.answer", kind = MessageLike)]
pub struct CallAnswerEventContent {
    /// The VoIP session description object.
    pub answer: AnswerSessionDescription,

    #[cfg(not(feature = "unstable-msc2746"))]
    /// A unique identifier for the call.
    ///
    /// With the `unstable-msc2746` feature, this uses the stricter `VoipId` type.
    pub call_id: String,

    #[cfg(feature = "unstable-msc2746")]
    /// A unique identifier for the call.
    ///
    /// Without the `unstable-msc2746` feature, this can be any string.
    pub call_id: OwnedVoipId,

    #[cfg(feature = "unstable-msc2746")]
    /// **Required in version 1.** A unique ID for this session for the duration of the call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party_id: Option<OwnedVoipId>,

    #[cfg(not(feature = "unstable-msc2746"))]
    /// The version of the VoIP specification this messages adheres to.
    ///
    /// With the `unstable-msc2746` feature, this can be a `UInt` or a `String`.
    pub version: UInt,

    #[cfg(feature = "unstable-msc2746")]
    /// The version of the VoIP specification this messages adheres to.
    ///
    /// Without the `unstable-msc2746` feature, this is a `UInt`.
    pub version: CallVersion,

    #[cfg(feature = "unstable-msc2746")]
    /// **Added in version 1.** The VoIP capabilities of the client.
    #[serde(default, skip_serializing_if = "CallCapabilities::is_default")]
    pub capabilities: CallCapabilities,
}

impl CallAnswerEventContent {
    /// Creates an `CallAnswerEventContent` with the given answer, call ID and VoIP version.
    ///
    /// With the `unstable-msc2746` feature, this method takes an `OwnedVoipId` for the call ID and
    /// a `CallVersion` for the version.
    #[cfg(not(feature = "unstable-msc2746"))]
    pub fn new(answer: AnswerSessionDescription, call_id: String, version: UInt) -> Self {
        Self { answer, call_id, version }
    }

    /// Creates an `CallAnswerEventContent` with the given answer, call ID and VoIP version.
    ///
    /// Without the `unstable-msc2746` feature, this method takes a `String` for the call ID and a
    /// `UInt` for the version.
    #[cfg(feature = "unstable-msc2746")]
    pub fn new(
        answer: AnswerSessionDescription,
        call_id: OwnedVoipId,
        version: CallVersion,
    ) -> Self {
        Self { answer, call_id, party_id: None, version, capabilities: Default::default() }
    }

    /// Convenience method to create a version 0 `CallAnswerEventContent` with all the required
    /// fields.
    #[cfg(feature = "unstable-msc2746")]
    pub fn version_0(answer: AnswerSessionDescription, call_id: OwnedVoipId) -> Self {
        Self::new(answer, call_id, uint!(0).into())
    }

    /// Convenience method to create a version 1 `CallAnswerEventContent` with all the required
    /// fields.
    #[cfg(feature = "unstable-msc2746")]
    pub fn version_1(
        answer: AnswerSessionDescription,
        call_id: OwnedVoipId,
        party_id: OwnedVoipId,
        capabilities: CallCapabilities,
    ) -> Self {
        Self { answer, call_id, party_id: Some(party_id), version: uint!(1).into(), capabilities }
    }
}
