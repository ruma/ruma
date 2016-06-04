//! Types for the *m.call.invite* event.

use core::EventType;
use super::SessionDescription;

/// This event is sent by the caller when they wish to establish a call.
#[derive(Debug, Deserialize, Serialize)]
pub struct InviteEvent {
    pub content: InviteEventContent,
    pub event_id: String,
    pub event_type: EventType,
    pub room_id: String,
    pub user_id: String,
}

/// The payload of an `InviteEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct InviteEventContent {
    /// A unique identifer for the call.
    pub call_id: String,
    /// The time in milliseconds that the invite is valid for. Once the invite age exceeds this
    /// value, clients should discard it. They should also no longer show the call as awaiting an
    /// answer in the UI.
    pub lifetime: u64,
    /// The session description object.
    pub offer: SessionDescription,
    /// The version of the VoIP specification this messages adheres to.
    pub version: u64,
}
