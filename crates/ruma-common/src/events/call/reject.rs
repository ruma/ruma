//! Types for the `m.call.reject` event [MSC2746].
//!
//! [MSC2746]: https://github.com/matrix-org/matrix-spec-proposals/pull/2746

use js_int::uint;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::CallVersion;
use crate::OwnedVoipId;

/// **Added in version 1.** The content of an `m.call.reject` event.
///
/// Starting from version 1, this event is sent by the callee to reject an invite.
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
    /// Cannot be less than `1`.
    pub version: CallVersion,
}

impl CallRejectEventContent {
    /// Creates a `CallRejectEventContent` with the given call ID, VoIP version and party ID.
    pub fn new(call_id: OwnedVoipId, party_id: OwnedVoipId, version: CallVersion) -> Self {
        Self { call_id, party_id, version }
    }

    /// Convenience method to create a version 1 `CallRejectEventContent` with all the required
    /// fields.
    pub fn version_1(call_id: OwnedVoipId, party_id: OwnedVoipId) -> Self {
        Self::new(call_id, party_id, uint!(1).into())
    }
}
