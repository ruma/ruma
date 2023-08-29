//! Types for the [`m.call.reject`] event.
//!
//! [`m.call.reject`]: https://spec.matrix.org/latest/client-server-api/#mcallreject

use ruma_common::{OwnedVoipId, VoipVersionId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// **Added in VoIP version 1.** The content of an `m.call.reject` event.
///
/// Starting from VoIP version 1, this event is sent by the callee to reject an invite.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.call.reject", kind = MessageLike)]
pub struct CallRejectEventContent {
    /// The ID of the call this event relates to.
    pub call_id: OwnedVoipId,

    /// A unique ID for this session for the duration of the call.
    pub party_id: OwnedVoipId,

    /// The version of the VoIP specification this messages adheres to.
    ///
    /// Cannot be older than `VoipVersionId::V1`.
    pub version: VoipVersionId,
}

impl CallRejectEventContent {
    /// Creates a `CallRejectEventContent` with the given call ID, VoIP version and party ID.
    pub fn new(call_id: OwnedVoipId, party_id: OwnedVoipId, version: VoipVersionId) -> Self {
        Self { call_id, party_id, version }
    }

    /// Convenience method to create a version 1 `CallRejectEventContent` with all the required
    /// fields.
    pub fn version_1(call_id: OwnedVoipId, party_id: OwnedVoipId) -> Self {
        Self::new(call_id, party_id, VoipVersionId::V1)
    }
}
