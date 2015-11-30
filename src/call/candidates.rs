//! Types for the *m.call.candidates* event.

use core::{Event, RoomEvent};

/// This event is sent by callers after sending an invite and by the callee after answering.
/// Its purpose is to give the other party additional ICE candidates to try using to communicate.
pub struct CandidatesEvent<'a> {
    content: CandidatesEventContent<'a>,
    event_id: &'a str,
    room_id: &'a str,
    user_id: &'a str,
}

impl<'a> Event<'a, CandidatesEventContent<'a>> for CandidatesEvent<'a> {
    fn content(&'a self) -> &'a CandidatesEventContent {
        &self.content
    }

    fn event_type(&self) -> &'static str {
        "m.call.candidates"
    }
}

impl<'a> RoomEvent<'a, CandidatesEventContent<'a>> for CandidatesEvent<'a> {
    fn event_id(&'a self) -> &'a str {
        &self.event_id
    }

    fn room_id(&'a self) -> &'a str {
        &self.room_id
    }

    fn user_id(&'a self) -> &'a str {
        &self.user_id
    }
}

/// The payload of a `CandidatesEvent`.
pub struct CandidatesEventContent<'a> {
    /// The ID of the call this event relates to.
    call_id: &'a str,
    /// A list of candidates.
    candidates: &'a[Candidate<'a>],
    /// The version of the VoIP specification this messages adheres to.
    version: u64,
}

/// An ICE (Interactive Connectivity Establishment) candidate.
pub struct Candidate<'a> {
    /// The SDP "a" line of the candidate.
    candidate: &'a str,
    /// The SDP media type this candidate is intended for.
    sdp_mid: &'a str,
    /// The index of the SDP "m" line this candidate is intended for.
    sdp_m_line_index: u64,
}
