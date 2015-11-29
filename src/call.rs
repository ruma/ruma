//! Events within the *m.call* namespace.

use core::{Event, RoomEvent};

/// This event is sent by the callee when they wish to answer the call.
pub struct AnswerEvent<'a> {
    content: AnswerEventContent<'a>,
    event_id: String,
    room_id: String,
    user_id: String,
}

impl<'a> Event<'a, AnswerEventContent<'a>> for AnswerEvent<'a> {
    fn content(&'a self) -> &'a AnswerEventContent {
        &self.content
    }

    fn event_type(&self) -> &'static str {
        "m.call.answer"
    }
}

impl<'a> RoomEvent<'a, AnswerEventContent<'a>> for AnswerEvent<'a> {
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

/// The payload of an `AnswerEvent`.
pub struct AnswerEventContent<'a> {
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

/// Sent by either party to signal their termination of the call. This can be sent either once the
/// call has has been established or before to abort the call.
pub struct HangupEvent<'a> {
    content: HangupEventContent<'a>,
    event_id: &'a str,
    room_id: &'a str,
    user_id: &'a str,
}

impl<'a> Event<'a, HangupEventContent<'a>> for HangupEvent<'a> {
    fn content(&'a self) -> &'a HangupEventContent {
        &self.content
    }

    fn event_type(&self) -> &'static str {
        "m.call.hangup"
    }
}

impl<'a> RoomEvent<'a, HangupEventContent<'a>> for HangupEvent<'a> {
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

/// The payload of a `HangupEvent`.
pub struct HangupEventContent<'a> {
    /// The ID of the call this event relates to.
    call_id: &'a str,
    /// The version of the VoIP specification this messages adheres to.
    version: u64,
}

/// This event is sent by the caller when they wish to establish a call.
pub struct InviteEvent<'a> {
    content: InviteEventContent<'a>,
    event_id: &'a str,
    room_id: &'a str,
    user_id: &'a str,
}

impl<'a> Event<'a, InviteEventContent<'a>> for InviteEvent<'a> {
    fn content(&'a self) -> &'a InviteEventContent {
        &self.content
    }

    fn event_type(&self) -> &'static str {
        "m.call.invite"
    }
}

impl<'a> RoomEvent<'a, InviteEventContent<'a>> for InviteEvent<'a> {
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

/// The payload of an `InviteEvent`.
pub struct InviteEventContent<'a> {
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
