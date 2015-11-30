//! Types for the *m.call.hangup* event.

use core::{Event, RoomEvent};

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
