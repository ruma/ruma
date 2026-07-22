//! Types for the `m.room.presence_sharing` state event.
//!
//! This event uses the unstable prefix defined in [MSC4495].
//!
//! [MSC4495]: https://github.com/matrix-org/matrix-spec-proposals/pull/4495

use ruma_macros::{EventContent, StringEnum};
use serde::{Deserialize, Serialize};

use crate::{EmptyStateKey, PrivOwnedStr};

/// A room's presence sharing hint.
///
/// The room's presence sharing hint indicates either:
/// 1. Sharing presence within the room is prohibited ([Self::Forbid]), or
/// 2. Sharing presence within the room should be recommended by clients ([Self::Suggest])
///
/// This type is defined in [MSC4495].
///
/// [MSC4495]: https://github.com/matrix-org/matrix-spec-proposals/pull/4495
#[derive(Clone, StringEnum)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_enum(rename_all = "snake_case")]
pub enum PresenceSharingHint {
    /// Sharing presence within the room is prohibited.
    Forbid,

    /// Sharing presence within the room should be recommended by clients.
    Suggest,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The content of an `m.room.presence_sharing` event.
///
/// The room's presence sharing hint indicates either:
/// 1. Sharing presence within the room is prohibited (`forbid`), or
/// 2. Sharing presence within the room should be recommended by clients (`suggest`)
///
/// This event uses the unstable prefix defined in [MSC4495].
///
/// [MSC4495]: https://github.com/matrix-org/matrix-spec-proposals/pull/4495
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "org.continuwuity.presence_v2.msc4495.room.presence_sharing", kind = State, state_key_type = EmptyStateKey)]
pub struct RoomPresenceSharingEventContent {
    /// The room's presence sharing hint.
    pub presence_sharing: PresenceSharingHint,
}

impl RoomPresenceSharingEventContent {
    /// Creates a new `RoomPresenceSharingEventContent` with the given hint.
    pub fn new(hint: PresenceSharingHint) -> Self {
        Self { presence_sharing: hint }
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::canonical_json::assert_to_canonical_json_eq;
    use serde_json::{from_value as from_json_value, json};

    use super::{PresenceSharingHint, RoomPresenceSharingEventContent};
    use crate::OriginalStateEvent;

    #[test]
    fn serialization() {
        let content =
            RoomPresenceSharingEventContent { presence_sharing: PresenceSharingHint::Forbid };

        assert_to_canonical_json_eq!(
            content,
            json!({
                "presence_sharing": "forbid",
            }),
        );
    }

    #[test]
    fn deserialization() {
        let json_data = json!({
            "content": {
                "presence_sharing": "suggest"
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "org.continuwuity.presence_v2.msc4495.room.presence_sharing"
        });

        assert_eq!(
            from_json_value::<OriginalStateEvent<RoomPresenceSharingEventContent>>(json_data)
                .unwrap()
                .content
                .presence_sharing,
            PresenceSharingHint::Suggest
        );
    }
}
