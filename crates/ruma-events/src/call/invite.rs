//! Types for the [`m.call.invite`] event.
//!
//! [`m.call.invite`]: https://spec.matrix.org/latest/client-server-api/#mcallinvite

use std::collections::BTreeMap;

use js_int::UInt;
use ruma_common::{OwnedUserId, OwnedVoipId, VoipVersionId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

#[cfg(feature = "unstable-msc2747")]
use super::CallCapabilities;
use super::{SessionDescription, StreamMetadata};

/// The content of an `m.call.invite` event.
///
/// This event is sent by the caller when they wish to establish a call.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.call.invite", kind = MessageLike)]
pub struct CallInviteEventContent {
    /// A unique identifier for the call.
    pub call_id: OwnedVoipId,

    /// **Required in VoIP version 1.** A unique ID for this session for the duration of the call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party_id: Option<OwnedVoipId>,

    /// The time in milliseconds that the invite is valid for.
    ///
    /// Once the invite age exceeds this value, clients should discard it. They should also no
    /// longer show the call as awaiting an answer in the UI.
    pub lifetime: UInt,

    /// The session description object.
    pub offer: SessionDescription,

    /// The version of the VoIP specification this messages adheres to.
    pub version: VoipVersionId,

    #[cfg(feature = "unstable-msc2747")]
    /// The VoIP capabilities of the client.
    #[serde(default, skip_serializing_if = "CallCapabilities::is_default")]
    pub capabilities: CallCapabilities,

    /// **Added in VoIP version 1.** The intended target of the invite, if any.
    ///
    /// If this is `None`, the invite is intended for any member of the room, except the sender.
    ///
    /// The invite should be ignored if the invitee is set and doesn't match the user's ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invitee: Option<OwnedUserId>,

    /// **Added in VoIP version 1.** Metadata describing the streams that will be sent.
    ///
    /// This is a map of stream ID to metadata about the stream.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sdp_stream_metadata: BTreeMap<String, StreamMetadata>,
}

impl CallInviteEventContent {
    /// Creates a new `CallInviteEventContent` with the given call ID, lifetime, offer and VoIP
    /// version.
    pub fn new(
        call_id: OwnedVoipId,
        lifetime: UInt,
        offer: SessionDescription,
        version: VoipVersionId,
    ) -> Self {
        Self {
            call_id,
            party_id: None,
            lifetime,
            offer,
            version,
            #[cfg(feature = "unstable-msc2747")]
            capabilities: Default::default(),
            invitee: None,
            sdp_stream_metadata: Default::default(),
        }
    }

    /// Convenience method to create a version 0 `CallInviteEventContent` with all the required
    /// fields.
    pub fn version_0(call_id: OwnedVoipId, lifetime: UInt, offer: SessionDescription) -> Self {
        Self::new(call_id, lifetime, offer, VoipVersionId::V0)
    }

    /// Convenience method to create a version 1 `CallInviteEventContent` with all the required
    /// fields.
    pub fn version_1(
        call_id: OwnedVoipId,
        party_id: OwnedVoipId,
        lifetime: UInt,
        offer: SessionDescription,
    ) -> Self {
        Self {
            call_id,
            party_id: Some(party_id),
            lifetime,
            offer,
            version: VoipVersionId::V1,
            #[cfg(feature = "unstable-msc2747")]
            capabilities: Default::default(),
            invitee: None,
            sdp_stream_metadata: Default::default(),
        }
    }
}
