//! Types for the [`m.call.sdp_stream_metadata_changed`] event.
//!
//! [`m.call.sdp_stream_metadata_changed`]: https://github.com/matrix-org/matrix-spec-proposals/pull/3291

use std::collections::BTreeMap;

use ruma_common::{OwnedVoipId, VoipVersionId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::StreamMetadata;

/// The content of an `m.call.sdp_stream_metadata_changed` event.
///
/// This event is sent by any party when a stream metadata changes but no negotiation is required.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.call.sdp_stream_metadata_changed", kind = MessageLike)]
pub struct CallSdpStreamMetadataChangedEventContent {
    /// A unique identifier for the call.
    pub call_id: OwnedVoipId,

    /// A unique ID for this session for the duration of the call.
    pub party_id: OwnedVoipId,

    /// The version of the VoIP specification this messages adheres to.
    ///
    /// Must be at least [`VoipVersionId::V1`].
    pub version: VoipVersionId,

    /// Metadata describing the streams that will be sent.
    ///
    /// This is a map of stream ID to metadata about the stream.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sdp_stream_metadata: BTreeMap<String, StreamMetadata>,
}

impl CallSdpStreamMetadataChangedEventContent {
    /// Creates a new `SdpStreamMetadataChangedEventContent` with the given call ID, party ID, VoIP
    /// version and stream metadata.
    pub fn new(
        call_id: OwnedVoipId,
        party_id: OwnedVoipId,
        version: VoipVersionId,
        sdp_stream_metadata: BTreeMap<String, StreamMetadata>,
    ) -> Self {
        Self { call_id, party_id, version, sdp_stream_metadata }
    }
}
