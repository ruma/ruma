//! Types for the *m.call.candidates* event.

room_event! {
    /// This event is sent by callers after sending an invite and by the callee after answering.
    /// Its purpose is to give the other party additional ICE candidates to try using to
    /// communicate.
    pub struct CandidatesEvent(CandidatesEventContent) {}
}

/// The payload of a `CandidatesEvent`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CandidatesEventContent {
    /// The ID of the call this event relates to.
    pub call_id: String,
    /// A list of candidates.
    pub candidates: Vec<Candidate>,
    /// The version of the VoIP specification this messages adheres to.
    pub version: u64,
}

/// An ICE (Interactive Connectivity Establishment) candidate.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Candidate {
    /// The SDP "a" line of the candidate.
    pub candidate: String,
    /// The SDP media type this candidate is intended for.
    pub sdp_mid: String,
    /// The index of the SDP "m" line this candidate is intended for.
    pub sdp_m_line_index: u64,
}
