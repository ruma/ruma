//! Events within the *m.call* namespace.

use core::{Event, RoomEvent};

/// This event is sent by the callee when they wish to answer the call.
pub struct Answer<'a> {
    content: AnswerContent<'a>,
    event_id: String,
    room_id: String,
    user_id: String,
}

impl<'a> Event<'a, AnswerContent<'a>> for Answer<'a> {
    fn content(&'a self) -> &'a AnswerContent {
        &self.content
    }

    fn event_type(&self) -> &'static str {
        "m.call.answer"
    }
}

impl<'a> RoomEvent<'a, AnswerContent<'a>> for Answer<'a> {
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

/// The payload of an `Answer` event.
pub struct AnswerContent<'a> {
    /// The VoIP session description.
    answer: SessionDescription<'a>,
    /// The ID of the call this event relates to.
    call_id: String,
    /// The version of the VoIP specification this messages adheres to.
    version: u64,
}

/// A VoIP session description.
pub struct SessionDescription<'a> {
    /// The type of session description.
    session_type: SessionDescriptionType,
    /// The SDP text of the session description.
    sdp: &'a str,
}

/// The type of VoIP session description.
pub enum SessionDescriptionType {
    /// An answer.
    Answer,
    /// An offer.
    Offer,
}

/// This event is sent by callers after sending an invite and by the callee after answering.
/// Its purpose is to give the other party additional ICE candidates to try using to communicate.
pub struct Candidates<'a> {
    content: CandidatesContent<'a>,
    event_id: &'a str,
    room_id: &'a str,
    user_id: &'a str,
}

impl<'a> Event<'a, CandidatesContent<'a>> for Candidates<'a> {
    fn content(&'a self) -> &'a CandidatesContent {
        &self.content
    }

    fn event_type(&self) -> &'static str {
        "m.call.candidates"
    }
}

impl<'a> RoomEvent<'a, CandidatesContent<'a>> for Candidates<'a> {
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

/// The payload of a `Candidates` event.
pub struct CandidatesContent<'a> {
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

/// Sent by either party to signal their termination of the call. This can be sent either once the
/// call has has been established or before to abort the call.
pub struct Hangup<'a> {
    content: HangupContent<'a>,
    event_id: &'a str,
    room_id: &'a str,
    user_id: &'a str,
}

impl<'a> Event<'a, HangupContent<'a>> for Hangup<'a> {
    fn content(&'a self) -> &'a HangupContent {
        &self.content
    }

    fn event_type(&self) -> &'static str {
        "m.call.hangup"
    }
}

impl<'a> RoomEvent<'a, HangupContent<'a>> for Hangup<'a> {
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

/// The payload of a `Hangup` event.
pub struct HangupContent<'a> {
    /// The ID of the call this event relates to.
    call_id: &'a str,
    /// The version of the VoIP specification this messages adheres to.
    version: u64,
}

/// This event is sent by the caller when they wish to establish a call.
pub struct Invite<'a> {
    content: InviteContent<'a>,
    event_id: &'a str,
    room_id: &'a str,
    user_id: &'a str,
}

impl<'a> Event<'a, InviteContent<'a>> for Invite<'a> {
    fn content(&'a self) -> &'a InviteContent {
        &self.content
    }

    fn event_type(&self) -> &'static str {
        "m.call.invite"
    }
}

impl<'a> RoomEvent<'a, InviteContent<'a>> for Invite<'a> {
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

/// The payload of an `Invite` event.
pub struct InviteContent<'a> {
    /// A unique identifer for the call.
    call_id: &'a str,
    /// The time in milliseconds that the invite is valid for. Once the invite age exceeds this
    /// value, clients should discard it. They should also no longer show the call as awaiting an
    /// answer in the UI.
    lifetime: u64,
    /// The session description object.
    offer: SessionDescription<'a>,
    /// The version of the VoIP specification this messages adheres to.
    version: u64,
}
