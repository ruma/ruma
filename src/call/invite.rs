//! Types for the *m.call.invite* event.

use super::SessionDescription;

room_event! {
    /// This event is sent by the caller when they wish to establish a call.
    pub struct InviteEvent(InviteEventContent) {}
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
