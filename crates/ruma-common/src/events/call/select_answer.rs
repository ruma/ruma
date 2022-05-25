//! Types for the `m.call.select_answer` event [MSC2746].
//!
//! [MSC2746]: https://github.com/matrix-org/matrix-spec-proposals/pull/2746

use js_int::uint;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::CallVersion;
use crate::OwnedVoipId;

/// **Added in version 1.** The content of an `m.call.select_answer` event.
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
    /// Cannot be less than `1`.
    pub version: CallVersion,
}

impl CallSelectAnswerEventContent {
    /// Creates a `CallSelectAnswerEventContent` with the given call ID, VoIP version, party ID and
    /// selected party ID.
    pub fn new(
        call_id: OwnedVoipId,
        party_id: OwnedVoipId,
        selected_party_id: OwnedVoipId,
        version: CallVersion,
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
        Self::new(call_id, party_id, selected_party_id, uint!(1).into())
    }
}
