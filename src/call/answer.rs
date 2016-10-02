//! Types for the *m.call.answer* event.

use super::SessionDescription;

room_event! {
    /// This event is sent by the callee when they wish to answer the call.
    pub struct AnswerEvent(AnswerEventContent) {}
}

/// The payload of an `AnswerEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct AnswerEventContent {
    /// The VoIP session description.
    pub answer: SessionDescription,
    /// The ID of the call this event relates to.
    pub call_id: String,
    /// The version of the VoIP specification this messages adheres to.
    pub version: u64,
}
