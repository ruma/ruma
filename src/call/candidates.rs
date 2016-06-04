//! Types for the *m.call.candidates* event.

use core::EventType;

/// This event is sent by callers after sending an invite and by the callee after answering.
/// Its purpose is to give the other party additional ICE candidates to try using to communicate.
pub struct CandidatesEvent {
    content: CandidatesEventContent,
    event_id: String,
    event_type: EventType,
    room_id: String,
    user_id: String,
}

/// The payload of a `CandidatesEvent`.
pub struct CandidatesEventContent {
    /// The ID of the call this event relates to.
    call_id: String,
    /// A list of candidates.
    candidates: Vec<Candidate>,
    /// The version of the VoIP specification this messages adheres to.
    version: u64,
}

/// An ICE (Interactive Connectivity Establishment) candidate.
pub struct Candidate {
    /// The SDP "a" line of the candidate.
    candidate: String,
    /// The SDP media type this candidate is intended for.
    sdp_mid: String,
    /// The index of the SDP "m" line this candidate is intended for.
    sdp_m_line_index: u64,
}
