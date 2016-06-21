//! Types for the *m.call.answer* event.

use events::EventType;
use super::SessionDescription;

/// This event is sent by the callee when they wish to answer the call.
#[derive(Debug, Deserialize, Serialize)]
pub struct AnswerEvent {
    pub content: AnswerEventContent,
    pub event_id: String,
    pub event_type: EventType,
    pub room_id: String,
    pub user_id: String,
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
