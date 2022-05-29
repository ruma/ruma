//! Types for the [`m.call.candidates`] event.
//!
//! [`m.call.candidates`]: https://spec.matrix.org/v1.2/client-server-api/#mcallcandidates

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{OwnedVoipId, VoipVersionId};

/// The content of an `m.call.candidates` event.
///
/// This event is sent by callers after sending an invite and by the callee after answering. Its
/// purpose is to give the other party additional ICE candidates to try using to communicate.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.call.candidates", kind = MessageLike)]
pub struct CallCandidatesEventContent {
    /// A unique identifier for the call.
    pub call_id: OwnedVoipId,

    #[cfg(feature = "unstable-msc2746")]
    /// **Required in VoIP version 1.** The unique ID for this session for the duration of the
    /// call.
    ///
    /// Must be the same as the one sent by the previous invite or answer from
    /// this session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party_id: Option<OwnedVoipId>,

    /// A list of candidates.
    ///
    /// With the `unstable-msc2746` feature, in VoIP version 1, this list should end with a
    /// `Candidate` with an empty `candidate` field when no more candidates will be sent.
    pub candidates: Vec<Candidate>,

    /// The version of the VoIP specification this messages adheres to.
    pub version: VoipVersionId,
}

impl CallCandidatesEventContent {
    /// Creates a new `CallCandidatesEventContent` with the given call id, candidate list and VoIP
    /// version.
    pub fn new(call_id: OwnedVoipId, candidates: Vec<Candidate>, version: VoipVersionId) -> Self {
        Self {
            call_id,
            candidates,
            version,
            #[cfg(feature = "unstable-msc2746")]
            party_id: None,
        }
    }

    /// Convenience method to create a VoIP version 0 `CallCandidatesEventContent` with all the
    /// required fields.
    pub fn version_0(call_id: OwnedVoipId, candidates: Vec<Candidate>) -> Self {
        Self::new(call_id, candidates, VoipVersionId::V0)
    }

    /// Convenience method to create a VoIP version 1 `CallCandidatesEventContent` with all the
    /// required fields.
    #[cfg(feature = "unstable-msc2746")]
    pub fn version_1(
        call_id: OwnedVoipId,
        party_id: OwnedVoipId,
        candidates: Vec<Candidate>,
    ) -> Self {
        Self { call_id, party_id: Some(party_id), candidates, version: VoipVersionId::V1 }
    }
}

/// An ICE (Interactive Connectivity Establishment) candidate.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    /// The SDP "a" line of the candidate.
    pub candidate: String,

    /// The SDP media type this candidate is intended for.
    pub sdp_mid: String,

    /// The index of the SDP "m" line this candidate is intended for.
    pub sdp_m_line_index: UInt,
}

impl Candidate {
    /// Creates a new `Candidate` with the given "a" line, SDP media type and SDP "m" line.
    pub fn new(candidate: String, sdp_mid: String, sdp_m_line_index: UInt) -> Self {
        Self { candidate, sdp_mid, sdp_m_line_index }
    }
}
