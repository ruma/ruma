//! Types for the *m.call.invite* event.

use core::{Event, RoomEvent};
use super::{SessionDescription, SessionDescriptionType};

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
