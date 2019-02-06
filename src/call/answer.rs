//! Types for the *m.call.answer* event.

use serde::{Deserialize, Serialize};

use super::SessionDescription;

room_event! {
    /// This event is sent by the callee when they wish to answer the call.
    pub struct AnswerEvent(AnswerEventContent) {}
}

/// The payload of an `AnswerEvent`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AnswerEventContent {
    /// The VoIP session description object. The session description type must be *answer*.
    pub answer: SessionDescription,
    /// The ID of the call this event relates to.
    pub call_id: String,
    /// The version of the VoIP specification this messages adheres to.
    pub version: u64,
}
