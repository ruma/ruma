//! Types for the [`m.room.join_rules`] event.
//!
//! [`m.room.join_rules`]: https://spec.matrix.org/latest/client-server-api/#mroomjoin_rules

pub use ruma_common::room::{AllowRule, JoinRule, Restricted};
use ruma_macros::EventContent;
use serde::{de, Deserialize, Serialize};

use crate::EmptyStateKey;

/// The content of an `m.room.join_rules` event.
///
/// Describes how users are allowed to join the room.
#[derive(Clone, Debug, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.room.join_rules", kind = State, state_key_type = EmptyStateKey)]
pub struct RoomJoinRulesEventContent {
    /// The type of rules used for users wishing to join this room.
    #[ruma_event(skip_redaction)]
    #[serde(flatten)]
    pub join_rule: JoinRule,
}

impl RoomJoinRulesEventContent {
    /// Creates a new `RoomJoinRulesEventContent` with the given rule.
    pub fn new(join_rule: JoinRule) -> Self {
        Self { join_rule }
    }

    /// Creates a new `RoomJoinRulesEventContent` with the restricted rule and the given set of
    /// allow rules.
    pub fn restricted(allow: Vec<AllowRule>) -> Self {
        Self { join_rule: JoinRule::Restricted(Restricted::new(allow)) }
    }

    /// Creates a new `RoomJoinRulesEventContent` with the knock restricted rule and the given set
    /// of allow rules.
    pub fn knock_restricted(allow: Vec<AllowRule>) -> Self {
        Self { join_rule: JoinRule::KnockRestricted(Restricted::new(allow)) }
    }
}

impl<'de> Deserialize<'de> for RoomJoinRulesEventContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let join_rule = JoinRule::deserialize(deserializer)?;
        Ok(RoomJoinRulesEventContent { join_rule })
    }
}

impl RoomJoinRulesEvent {
    /// Obtain the join rule, regardless of whether this event is redacted.
    pub fn join_rule(&self) -> &JoinRule {
        match self {
            Self::Original(ev) => &ev.content.join_rule,
            Self::Redacted(ev) => &ev.content.join_rule,
        }
    }
}

impl SyncRoomJoinRulesEvent {
    /// Obtain the join rule, regardless of whether this event is redacted.
    pub fn join_rule(&self) -> &JoinRule {
        match self {
            Self::Original(ev) => &ev.content.join_rule,
            Self::Redacted(ev) => &ev.content.join_rule,
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::owned_room_id;

    use super::{AllowRule, JoinRule, OriginalSyncRoomJoinRulesEvent, RoomJoinRulesEventContent};

    #[test]
    fn deserialize() {
        let json = r#"{"join_rule": "public"}"#;
        let event: RoomJoinRulesEventContent = serde_json::from_str(json).unwrap();
        assert_matches!(event, RoomJoinRulesEventContent { join_rule: JoinRule::Public });
    }

    #[test]
    fn deserialize_restricted() {
        let json = r#"{
            "join_rule": "restricted",
            "allow": [
                {
                    "type": "m.room_membership",
                    "room_id": "!mods:example.org"
                },
                {
                    "type": "m.room_membership",
                    "room_id": "!users:example.org"
                }
            ]
        }"#;
        let event: RoomJoinRulesEventContent = serde_json::from_str(json).unwrap();
        assert_matches!(event.join_rule, JoinRule::Restricted(restricted));
        assert_eq!(
            restricted.allow,
            &[
                AllowRule::room_membership(owned_room_id!("!mods:example.org")),
                AllowRule::room_membership(owned_room_id!("!users:example.org"))
            ]
        );
    }

    #[test]
    fn deserialize_restricted_event() {
        let json = r#"{
            "type": "m.room.join_rules",
            "sender": "@admin:community.rs",
            "content": {
                "join_rule": "restricted",
                "allow": [
                    { "type": "m.room_membership","room_id": "!KqeUnzmXPIhHRaWMTs:mccarty.io" }
                ]
            },
            "state_key": "",
            "origin_server_ts":1630508835342,
            "unsigned": {
                "age":4165521871
            },
            "event_id": "$0ACb9KSPlT3al3kikyRYvFhMqXPP9ZcQOBrsdIuh58U"
        }"#;

        assert_matches!(serde_json::from_str::<OriginalSyncRoomJoinRulesEvent>(json), Ok(_));
    }

    #[test]
    fn restricted_room_no_allow_field() {
        let json = r#"{"join_rule":"restricted"}"#;
        let join_rules: RoomJoinRulesEventContent = serde_json::from_str(json).unwrap();
        assert_matches!(
            join_rules,
            RoomJoinRulesEventContent { join_rule: JoinRule::Restricted(_) }
        );
    }
}
