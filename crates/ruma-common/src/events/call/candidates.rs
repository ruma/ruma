//! Types for the [`m.call.candidates`] event.
//!
//! [`m.call.candidates`]: https://spec.matrix.org/v1.2/client-server-api/#mcallcandidates

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of an `m.call.candidates` event.
///
/// This event is sent by callers after sending an invite and by the callee after answering. Its
/// purpose is to give the other party additional ICE candidates to try using to communicate.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.call.candidates", kind = MessageLike)]
pub struct CallCandidatesEventContent {
    /// The ID of the call this event relates to.
    pub call_id: String,

    /// A list of candidates.
    pub candidates: Vec<Candidate>,

    /// The version of the VoIP specification this messages adheres to.
    pub version: UInt,
}

impl CallCandidatesEventContent {
    /// Creates a new `CandidatesEventContent` with the given call id, candidate list and VoIP
    /// version.
    pub fn new(call_id: String, candidates: Vec<Candidate>, version: UInt) -> Self {
        Self { call_id, candidates, version }
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
