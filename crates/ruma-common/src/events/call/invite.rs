//! Types for the [`m.call.invite`] event.
//!
//! [`m.call.invite`]: https://spec.matrix.org/v1.2/client-server-api/#mcallinvite

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::OfferSessionDescription;

/// The content of an `m.call.invite` event.
///
/// This event is sent by the caller when they wish to establish a call.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.call.invite", kind = MessageLike)]
pub struct CallInviteEventContent {
    /// A unique identifier for the call.
    pub call_id: String,

    /// The time in milliseconds that the invite is valid for.
    ///
    /// Once the invite age exceeds this value, clients should discard it. They should also no
    /// longer show the call as awaiting an answer in the UI.
    pub lifetime: UInt,

    /// The session description object.
    pub offer: OfferSessionDescription,

    /// The version of the VoIP specification this messages adheres to.
    pub version: UInt,
}

impl CallInviteEventContent {
    /// Creates a new `InviteEventContent` with the given call ID, lifetime and VoIP version.
    pub fn new(
        call_id: String,
        lifetime: UInt,
        offer: OfferSessionDescription,
        version: UInt,
    ) -> Self {
        Self { call_id, lifetime, offer, version }
    }
}
