//! Types for the *m.call.answer* event.

use js_int::UInt;
use ruma_events_macros::{FromRaw, MessageEventContent};
use serde::Serialize;

use super::SessionDescription;

/// This event is sent by the callee when they wish to answer the call.
#[derive(Clone, Debug, Serialize, FromRaw, MessageEventContent)]
#[ruma_event(type = "m.call.answer")]
pub struct AnswerEventContenet {
    /// The VoIP session description object. The session description type must be *answer*.
    pub answer: SessionDescription,

    /// The ID of the call this event relates to.
    pub call_id: String,

    /// The version of the VoIP specification this messages adheres to.
    pub version: UInt,
}
