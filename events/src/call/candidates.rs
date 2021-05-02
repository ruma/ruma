//! Types for the *m.call.candidates* event.

use js_int::UInt;
use ruma_events_macros::MessageEventContent;
use serde::{Deserialize, Serialize};

use crate::MessageEvent;

/// This event is sent by callers after sending an invite and by the callee after answering. Its
/// purpose is to give the other party additional ICE candidates to try using to communicate.
pub type CandidatesEvent = MessageEvent<CandidatesEventContent>;

/// The payload for `CandidatesEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, MessageEventContent)]
#[ruma_event(type = "m.call.candidates")]
pub struct CandidatesEventContent {
    /// The ID of the call this event relates to.
    pub call_id: String,

    /// A list of candidates.
    pub candidates: Vec<Candidate>,

    /// The version of the VoIP specification this messages adheres to.
    pub version: UInt,
}

/// An ICE (Interactive Connectivity Establishment) candidate.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    /// The SDP "a" line of the candidate.
    pub candidate: String,

    /// The SDP media type this candidate is intended for.
    pub sdp_mid: String,

    /// The index of the SDP "m" line this candidate is intended for.
    pub sdp_m_line_index: UInt,
}
