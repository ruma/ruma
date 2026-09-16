//! Types for the `m.presence.persistent` account data key.
//!
//! This uses the unstable prefix defined in [MSC4532].
//!
//! [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532

use ruma_common::presence::{PresenceState, PresenceStatus};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of the `m.presence.persistent` account data key.
///
/// This uses the unstable prefix defined in [MSC4532].
///
/// [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532
#[derive(Clone, Default, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "org.continuwuity.presence_v2.msc4532.presence.persistent", kind = GlobalAccountData)]
pub struct PresencePersistentEventContent {
    /// A persistent global override for the user's presence state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_override: Option<PresenceState>,

    /// The user's extensible status.
    #[serde(skip_serializing_if = "ruma_common::serde::is_default")]
    pub status: PresenceStatus,
}
