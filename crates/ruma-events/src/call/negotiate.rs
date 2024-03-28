//! Types for the [`m.call.negotiate`] event.
//!
//! [`m.call.negotiate`]: https://spec.matrix.org/latest/client-server-api/#mcallnegotiate

use std::collections::BTreeMap;

use js_int::UInt;
use ruma_common::{OwnedVoipId, VoipVersionId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{SessionDescription, StreamMetadata};

/// **Added in VoIP version 1.** The content of an `m.call.negotiate` event.
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

    /// The version of the VoIP specification this messages adheres to.
    pub version: VoipVersionId,

    /// The time in milliseconds that the negotiation is valid for.
    pub lifetime: UInt,

    /// The session description of the negotiation.
    pub description: SessionDescription,

    /// Metadata describing the streams that will be sent.
    ///
    /// This is a map of stream ID to metadata about the stream.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sdp_stream_metadata: BTreeMap<String, StreamMetadata>,
}

impl CallNegotiateEventContent {
    /// Creates a `CallNegotiateEventContent` with the given call ID, party ID, lifetime and
    /// description.
    pub fn new(
        call_id: OwnedVoipId,
        party_id: OwnedVoipId,
        version: VoipVersionId,
        lifetime: UInt,
        description: SessionDescription,
    ) -> Self {
        Self {
            call_id,
            party_id,
            version,
            lifetime,
            description,
            sdp_stream_metadata: Default::default(),
        }
    }

    /// Convenience method to create a version 1 `CallNegotiateEventContent` with all the required
    /// fields.
    pub fn version_1(
        call_id: OwnedVoipId,
        party_id: OwnedVoipId,
        lifetime: UInt,
        description: SessionDescription,
    ) -> Self {
        Self::new(call_id, party_id, VoipVersionId::V1, lifetime, description)
    }
}
