//! Types for the *m.call.invite* event.

use core::EventType;
use super::SessionDescription;

/// This event is sent by the caller when they wish to establish a call.
pub struct InviteEvent {
    content: InviteEventContent,
    event_id: String,
    event_type: EventType,
    room_id: String,
    user_id: String,
}

/// The payload of an `InviteEvent`.
pub struct InviteEventContent {
    /// A unique identifer for the call.
    call_id: String,
    /// The time in milliseconds that the invite is valid for. Once the invite age exceeds this
    /// value, clients should discard it. They should also no longer show the call as awaiting an
    /// answer in the UI.
    lifetime: u64,
    /// The session description object.
    offer: SessionDescription,
    /// The version of the VoIP specification this messages adheres to.
    version: u64,
}
