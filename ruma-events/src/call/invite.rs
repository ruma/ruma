//! Types for the *m.call.invite* event.

use js_int::UInt;
use ruma_events_macros::MessageEventContent;
use serde::{Deserialize, Serialize};

use super::SessionDescription;
use crate::MessageEvent;

/// This event is sent by the caller when they wish to establish a call.
pub type InviteEvent = MessageEvent<InviteEventContent>;

/// The payload for `InviteEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, MessageEventContent)]
#[ruma_event(type = "m.call.invite")]
pub struct InviteEventContent {
    /// A unique identifer for the call.
    pub call_id: String,

    /// The time in milliseconds that the invite is valid for. Once the invite age exceeds this
    /// value, clients should discard it. They should also no longer show the call as awaiting an
    /// answer in the UI.
    pub lifetime: UInt,

    /// The session description object. The session description type must be *offer*.
    pub offer: SessionDescription,

    /// The version of the VoIP specification this messages adheres to.
    pub version: UInt,
}
