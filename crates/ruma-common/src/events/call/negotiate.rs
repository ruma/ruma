//! Types for the `m.call.negotiate` event [MSC2746].
//!
//! [MSC2746]: https://github.com/matrix-org/matrix-spec-proposals/pull/2746

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::SessionDescription;
use crate::OwnedVoipId;

/// **Added in version 1.** The content of an `m.call.negotiate` event.
///
/// This event is sent by either party after the call is established to renegotiate it. It can be
/// used for media pause, hold/resume, ICE restarts and voice/video call up/downgrading.
///
/// First an event must be sent with an `offer` session description, which is replied to with an
/// event with an `answer` session description.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.call.negotiate", kind = MessageLike)]
pub struct CallNegotiateEventContent {
    /// The ID of the call this event relates to.
    pub call_id: OwnedVoipId,

    /// The unique ID for this session for the duration of the call.
    ///
    /// Must be the same as the one sent by the previous invite or answer from
    /// this session.
    pub party_id: OwnedVoipId,

    /// The time in milliseconds that the negotiation is valid for.
    pub lifetime: UInt,

    /// The session description of the negotiation.
    pub description: SessionDescription,
}

impl CallNegotiateEventContent {
    /// Creates a `CallNegotiateEventContent` with the given call ID, party ID, lifetime and
    /// description.
    pub fn new(
        call_id: OwnedVoipId,
        party_id: OwnedVoipId,
        lifetime: UInt,
        description: SessionDescription,
    ) -> Self {
        Self { call_id, party_id, lifetime, description }
    }
}
