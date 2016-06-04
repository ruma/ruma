//! Types for the *m.call.answer* event.

use core::EventType;
use super::SessionDescription;

/// This event is sent by the callee when they wish to answer the call.
pub struct AnswerEvent {
    content: AnswerEventContent,
    event_id: String,
    event_type: EventType,
    room_id: String,
    user_id: String,
}

/// The payload of an `AnswerEvent`.
pub struct AnswerEventContent {
    /// The VoIP session description.
    answer: SessionDescription,
    /// The ID of the call this event relates to.
    call_id: String,
    /// The version of the VoIP specification this messages adheres to.
    version: u64,
}
