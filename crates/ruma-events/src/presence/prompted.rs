//! Types for the `m.presence.prompted` account data key.
//!
//! This uses the unstable prefix defined in [MSC4495].
//!
//! [MSC4495]: https://github.com/matrix-org/matrix-spec-proposals/pull/4495

use ruma_common::{OwnedRoomId, OwnedUserId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of the `m.presence.prompted` account data key.
///
/// This uses the unstable prefix defined in [MSC4495].
///
/// [MSC4495]: https://github.com/matrix-org/matrix-spec-proposals/pull/4495
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "org.continuwuity.presence_v2.msc4495.presence.prompted", kind = GlobalAccountData)]
pub struct PresencePromptedEventContent {
    /// The list of users that have previously had the presence sharing prompt displayed.
    pub users: Vec<OwnedUserId>,

    /// The list of rooms that have previously had the presence sharing prompt displayed.
    pub rooms: Vec<OwnedRoomId>,
}

impl PresencePromptedEventContent {
    /// Creates a new `PresencePromptedEventContent` with the given users and rooms.
    pub fn new(users: Vec<OwnedUserId>, rooms: Vec<OwnedRoomId>) -> Self {
        Self { users, rooms }
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::{canonical_json::assert_to_canonical_json_eq, owned_room_id, owned_user_id};
    use serde_json::{from_value as from_json_value, json};

    use super::PresencePromptedEventContent;

    #[test]
    fn serialization() {
        let content = PresencePromptedEventContent {
            users: vec![
                owned_user_id!("@alice:example.com"),
                owned_user_id!("@mallory:example.com"),
            ],
            rooms: vec![owned_room_id!("!family-group-chat")],
        };

        assert_to_canonical_json_eq!(
            content,
            json!({
                "users": [
                    "@alice:example.com",
                    "@mallory:example.com",
                ],
                "rooms": [
                    "!family-group-chat",
                ],
            }),
        );
    }

    #[test]
    fn deserialization() {
        let json_data = json!({
            "users": [
                "@alice:example.com",
                "@mallory:example.com",
            ],
            "rooms": [
                "!family-group-chat",
            ],
        });

        let content = from_json_value::<PresencePromptedEventContent>(json_data).unwrap();

        assert_eq!(content.users[0], "@alice:example.com");
        assert_eq!(content.users[1], "@mallory:example.com");
        assert_eq!(content.rooms[0], "!family-group-chat");
    }
}
