//! Types for the [`m.call.select_answer`] event.
//!
//! [`m.call.select_answer`]: https://spec.matrix.org/latest/client-server-api/#mcallselect_answer

use ruma_common::{OwnedVoipId, VoipVersionId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// **Added in VoIP version 1.** The content of an `m.call.select_answer` event.
///
/// This event is sent by the caller when it has chosen an answer.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.call.select_answer", kind = MessageLike)]
pub struct CallSelectAnswerEventContent {
    /// The ID of the call this event relates to.
    pub call_id: OwnedVoipId,

    /// A unique ID for this session for the duration of the call.
    ///
    /// Must be the same as the one sent by the previous invite from this session.
    pub party_id: OwnedVoipId,

    /// The party ID of the selected answer to the previously sent invite.
    pub selected_party_id: OwnedVoipId,

    /// The version of the VoIP specification this messages adheres to.
    ///
    /// Cannot be older than `VoipVersionId::V1`.
    pub version: VoipVersionId,
}

impl CallSelectAnswerEventContent {
    /// Creates a `CallSelectAnswerEventContent` with the given call ID, VoIP version, party ID and
    /// selected party ID.
    pub fn new(
        call_id: OwnedVoipId,
        party_id: OwnedVoipId,
        selected_party_id: OwnedVoipId,
        version: VoipVersionId,
    ) -> Self {
        Self { call_id, party_id, selected_party_id, version }
    }

    /// Convenience method to create a version 1 `CallSelectAnswerEventContent` with all the
    /// required fields.
    pub fn version_1(
        call_id: OwnedVoipId,
        party_id: OwnedVoipId,
        selected_party_id: OwnedVoipId,
    ) -> Self {
        Self::new(call_id, party_id, selected_party_id, VoipVersionId::V1)
    }
}
