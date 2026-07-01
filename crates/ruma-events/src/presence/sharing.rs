//! Types for the `m.presence.sharing` account data key.
//!
//! This uses the unstable prefix defined in [MSC4495].
//!
//! [MSC4495]: https://github.com/matrix-org/matrix-spec-proposals/pull/4495

use std::collections::BTreeMap;

use ruma_common::{OwnedRoomId, OwnedServerName, OwnedUserId};
use ruma_macros::{EventContent, StringEnum};
use serde::{Deserialize, Serialize};

use crate::PrivOwnedStr;

/// A possible state for a user in the `m.presence.sharing` configuration.
#[derive(Clone, StringEnum)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_enum(rename_all = "snake_case")]
pub enum UserPresenceSharingState {
    /// The user may receive presence updates.
    Allow,

    /// The user must not receive presence updates.
    Deny,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// A possible state for a room in the `m.presence.sharing` configuration.
#[derive(Clone, StringEnum)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_enum(rename_all = "snake_case")]
pub enum RoomPresenceSharingState {
    /// The room may receive presence updates.
    Allow,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// A possible state for a server in the `m.presence.sharing` configuration.
#[derive(Clone, StringEnum)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_enum(rename_all = "snake_case")]
pub enum ServerPresenceSharingState {
    /// The server must not receive presence updates.
    Deny,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The content of the `m.presence.sharing` account data key.
///
/// This uses the unstable prefix defined in [MSC4495].
///
/// [MSC4495]: https://github.com/matrix-org/matrix-spec-proposals/pull/4495
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "org.continuwuity.presence_v2.msc4495.presence.sharing", kind = GlobalAccountData)]
pub struct PresenceSharingEventContent {
    /// Whether presence should be shared with all users on the local homeserver.
    pub share_locally: bool,

    /// Configuration for sharing presence with users.
    pub users: BTreeMap<OwnedUserId, UserPresenceSharingState>,

    /// Configuration for sharing presence with rooms.
    ///
    /// Sharing presence with rooms also depends on the room's presence sharing hint.
    pub rooms: BTreeMap<OwnedRoomId, RoomPresenceSharingState>,

    /// Configuration for sharing presence with servers.
    pub servers: BTreeMap<OwnedServerName, ServerPresenceSharingState>,
}

impl PresenceSharingEventContent {
    /// Creates a new `PresenceSharingEventContent` with the given parameters.
    pub fn new(
        share_locally: bool,
        users: BTreeMap<OwnedUserId, UserPresenceSharingState>,
        rooms: BTreeMap<OwnedRoomId, RoomPresenceSharingState>,
        servers: BTreeMap<OwnedServerName, ServerPresenceSharingState>,
    ) -> Self {
        Self { share_locally, users, rooms, servers }
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::{
        canonical_json::assert_to_canonical_json_eq, owned_room_id, owned_server_name,
        owned_user_id,
    };
    use serde_json::{from_value as from_json_value, json};

    use crate::presence::sharing::{
        PresenceSharingEventContent, RoomPresenceSharingState, ServerPresenceSharingState,
        UserPresenceSharingState,
    };

    #[test]
    fn serialization() {
        let content = PresenceSharingEventContent {
            share_locally: true,
            users: [
                (owned_user_id!("@alice:example.com"), UserPresenceSharingState::Allow),
                (owned_user_id!("@mallory:example.com"), UserPresenceSharingState::Deny),
            ]
            .into(),
            rooms: [(owned_room_id!("!family-group-chat"), RoomPresenceSharingState::Allow)].into(),
            servers: [(owned_server_name!("matrix.org"), ServerPresenceSharingState::Deny)].into(),
        };

        assert_to_canonical_json_eq!(
            content,
            json!({
                "share_locally": true,
                "users": {
                    "@alice:example.com": "allow",
                    "@mallory:example.com": "deny",
                },
                "rooms": {
                    "!family-group-chat": "allow",
                },
                "servers": {
                    "matrix.org": "deny",
                },
            }),
        );
    }

    #[test]
    fn deserialization() {
        let json_data = json!({
            "share_locally": true,
            "users": {
                "@alice:example.com": "allow",
                "@mallory:example.com": "deny",
            },
            "rooms": {
                "!family-group-chat": "allow",
            },
            "servers": {
                "matrix.org": "deny",
            }
        });

        let content = from_json_value::<PresenceSharingEventContent>(json_data).unwrap();

        assert!(content.share_locally);
        assert_eq!(content.users.len(), 2);
        assert_eq!(
            content.users.get("@alice:example.com").unwrap(),
            &UserPresenceSharingState::Allow
        );
        assert_eq!(
            content.users.get("@mallory:example.com").unwrap(),
            &UserPresenceSharingState::Deny
        );
        assert_eq!(content.rooms.len(), 1);
        assert_eq!(
            content.rooms.get("!family-group-chat").unwrap(),
            &RoomPresenceSharingState::Allow
        );
        assert_eq!(content.servers.len(), 1);
        assert_eq!(content.servers.get("matrix.org").unwrap(), &ServerPresenceSharingState::Deny);
    }
}
