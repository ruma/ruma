//! Types for the *m.call.answer* event.

use core::{Event, EventType, RoomEvent};
use super::{SessionDescription, SessionDescriptionType};

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

    fn event_type(&self) -> EventType {
        EventType::CallAnswer
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
