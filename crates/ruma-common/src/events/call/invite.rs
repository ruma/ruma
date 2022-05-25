//! Types for the [`m.call.invite`] event.
//!
//! [`m.call.invite`]: https://spec.matrix.org/v1.2/client-server-api/#mcallinvite

#[cfg(feature = "unstable-msc2746")]
use js_int::uint;
use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::OfferSessionDescription;
#[cfg(feature = "unstable-msc2746")]
use super::{CallCapabilities, CallVersion};
use crate::OwnedUserId;
#[cfg(feature = "unstable-msc2746")]
use crate::OwnedVoipId;

/// The content of an `m.call.invite` event.
///
/// This event is sent by the caller when they wish to establish a call.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.call.invite", kind = MessageLike)]
pub struct CallInviteEventContent {
    #[cfg(not(feature = "unstable-msc2746"))]
    /// A unique identifier for the call.
    ///
    /// With the `unstable-msc2746` feature, this uses the stricter `OwnedVoipId` type.
    pub call_id: String,

    #[cfg(feature = "unstable-msc2746")]
    /// A unique identifier for the call.
    ///
    /// Without the `unstable-msc2746` feature, this can be any string.
    pub call_id: OwnedVoipId,

    #[cfg(feature = "unstable-msc2746")]
    /// **Required in version 1.** A unique ID for this session for the duration of the call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party_id: Option<OwnedVoipId>,

    /// The time in milliseconds that the invite is valid for.
    ///
    /// Once the invite age exceeds this value, clients should discard it. They should also no
    /// longer show the call as awaiting an answer in the UI.
    pub lifetime: UInt,

    /// The session description object.
    pub offer: OfferSessionDescription,

    #[cfg(not(feature = "unstable-msc2746"))]
    /// The version of the VoIP specification this messages adheres to.
    ///
    /// With the `unstable-msc2746` feature, this can be a `UInt` or a `String`.
    pub version: UInt,

    #[cfg(feature = "unstable-msc2746")]
    /// The version of the VoIP specification this messages adheres to.
    ///
    /// Without the `unstable-msc2746` feature, this is a `UInt`.
    pub version: CallVersion,

    #[cfg(feature = "unstable-msc2746")]
    /// **Added in version 1.** The VoIP capabilities of the client.
    #[serde(default, skip_serializing_if = "CallCapabilities::is_default")]
    pub capabilities: CallCapabilities,

    #[cfg(feature = "unstable-msc2746")]
    /// **Added in version 1.** The intended target of the invite, if any.
    ///
    /// If this is `None`, the invite is intended for any member of the room, except the sender.
    ///
    /// The invite should be ignored if the invitee is set and doesn't match the user's ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invitee: Option<OwnedUserId>,
}

impl CallInviteEventContent {
    /// Creates a new `CallInviteEventContent` with the given call ID, lifetime, offer and VoIP
    /// version.
    ///
    /// With the `unstable-msc2746` feature, this method takes an `OwnedVoipId` for the call ID and
    /// a `CallVersion` for the version.
    #[cfg(not(feature = "unstable-msc2746"))]
    pub fn new(
        call_id: String,
        lifetime: UInt,
        offer: OfferSessionDescription,
        version: UInt,
    ) -> Self {
        Self { call_id, lifetime, offer, version }
    }

    /// Creates a new `CallInviteEventContent` with the given call ID, lifetime, offer and VoIP
    /// version.
    ///
    /// Without the `unstable-msc2746` feature, this method takes a `String` for the call ID and a
    /// `UInt` for the version.
    #[cfg(feature = "unstable-msc2746")]
    pub fn new(
        call_id: OwnedVoipId,
        lifetime: UInt,
        offer: OfferSessionDescription,
        version: CallVersion,
    ) -> Self {
        Self {
            call_id,
            party_id: None,
            lifetime,
            offer,
            version,
            capabilities: Default::default(),
            invitee: None,
        }
    }

    /// Convenience method to create a version 0 `CallInviteEventContent` with all the required
    /// fields.
    #[cfg(feature = "unstable-msc2746")]
    pub fn version_0(call_id: OwnedVoipId, lifetime: UInt, offer: OfferSessionDescription) -> Self {
        Self::new(call_id, lifetime, offer, uint!(0).into())
    }

    /// Convenience method to create a version 1 `CallInviteEventContent` with all the required
    /// fields.
    #[cfg(feature = "unstable-msc2746")]
    pub fn version_1(
        call_id: OwnedVoipId,
        party_id: OwnedVoipId,
        lifetime: UInt,
        offer: OfferSessionDescription,
        capabilities: CallCapabilities,
    ) -> Self {
        Self {
            call_id,
            party_id: Some(party_id),
            lifetime,
            offer,
            version: uint!(1).into(),
            capabilities,
            invitee: None,
        }
    }
}
