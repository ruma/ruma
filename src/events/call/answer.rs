//! Types for the *m.call.answer* event.

use events::RoomEvent;
use super::SessionDescription;

/// This event is sent by the callee when they wish to answer the call.
pub type AnswerEvent = RoomEvent<AnswerEventContent>;

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
