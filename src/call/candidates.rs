//! Types for the *m.call.candidates* event.

use core::EventType;

/// This event is sent by callers after sending an invite and by the callee after answering.
/// Its purpose is to give the other party additional ICE candidates to try using to communicate.
#[derive(Debug, Deserialize, Serialize)]
pub struct CandidatesEvent {
    pub content: CandidatesEventContent,
    pub event_id: String,
    pub event_type: EventType,
    pub room_id: String,
    pub user_id: String,
}

/// The payload of a `CandidatesEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct CandidatesEventContent {
    /// The ID of the call this event relates to.
    pub call_id: String,
    /// A list of candidates.
    pub candidates: Vec<Candidate>,
    /// The version of the VoIP specification this messages adheres to.
    pub version: u64,
}

/// An ICE (Interactive Connectivity Establishment) candidate.
#[derive(Debug, Deserialize, Serialize)]
pub struct Candidate {
    /// The SDP "a" line of the candidate.
    pub candidate: String,
    /// The SDP media type this candidate is intended for.
    pub sdp_mid: String,
    /// The index of the SDP "m" line this candidate is intended for.
    pub sdp_m_line_index: u64,
}
