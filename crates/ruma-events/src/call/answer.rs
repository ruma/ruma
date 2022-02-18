//! Types for the [`m.call.answer`] event.
//!
//! [`m.call.answer`]: https://spec.matrix.org/v1.2/client-server-api/#mcallanswer

use js_int::UInt;
use ruma_events_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::SessionDescription;

/// The content of an `m.call.answer` event.
///
/// This event is sent by the callee when they wish to answer the call.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.call.answer", kind = Message)]
pub struct CallAnswerEventContent {
    /// The VoIP session description object.
    ///
    /// The session description type must be *answer*.
    pub answer: SessionDescription,

    /// The ID of the call this event relates to.
    pub call_id: String,

    /// The version of the VoIP specification this messages adheres to.
    pub version: UInt,
}

impl CallAnswerEventContent {
    /// Creates an `AnswerEventContent` with the given answer, call ID and VoIP version.
    pub fn new(answer: SessionDescription, call_id: String, version: UInt) -> Self {
        Self { answer, call_id, version }
    }
}
