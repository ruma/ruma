//! Types for the the *m.push_rules* event.

use ruma_events_macros::BasicEventContent;
use serde::{Deserialize, Serialize};

use crate::BasicEvent;

/// Describes all push rules for a user.
pub type PushRulesEvent = BasicEvent<PushRulesEventContent>;

/// The payload for `PushRulesEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[ruma_event(type = "m.push_rules")]
pub struct PushRulesEventContent {
    /// The global ruleset.
    pub global: Ruleset,
}

pub use ruma_common::push::Action;

/// A push ruleset scopes a set of rules according to some criteria.
///
/// For example, some rules may only be applied for messages from a particular sender, a particular
/// room, or by default. The push ruleset contains the entire set of scopes and rules.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ruleset {
    /// These rules configure behavior for (unencrypted) messages that match certain patterns.
    pub content: Vec<PatternedPushRule>,

    /// These user-configured rules are given the highest priority.
    ///
    /// This field is named `override_` instead of `override` because the latter is a reserved
    /// keyword in Rust.
    #[serde(rename = "override")]
    pub override_: Vec<ConditionalPushRule>,

    /// These rules change the behavior of all messages for a given room.
    pub room: Vec<PushRule>,

    /// These rules configure notification behavior for messages from a specific Matrix user ID.
    pub sender: Vec<PushRule>,

    /// These rules are identical to override rules, but have a lower priority than `content`,
    /// `room` and `sender` rules.
    pub underride: Vec<ConditionalPushRule>,
}

/// A push rule is a single rule that states under what conditions an event should be passed onto a
/// push gateway and how the notification should be presented.
///
/// These rules are stored on the user's homeserver. They are manually configured by the user, who
/// can create and view them via the Client/Server API.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PushRule {
    /// Actions to determine if and how a notification is delivered for events matching this rule.
    pub actions: Vec<Action>,

    /// Whether this is a default rule, or has been set explicitly.
    pub default: bool,

    /// Whether the push rule is enabled or not.
    pub enabled: bool,

    /// The ID of this rule.
    pub rule_id: String,
}

/// Like `PushRule`, but with an additional `conditions` field.
///
/// Only applicable to underride and override rules.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConditionalPushRule {
    /// Actions to determine if and how a notification is delivered for events matching this rule.
    pub actions: Vec<Action>,

    /// Whether this is a default rule, or has been set explicitly.
    pub default: bool,

    /// Whether the push rule is enabled or not.
    pub enabled: bool,

    /// The ID of this rule.
    pub rule_id: String,

    /// The conditions that must hold true for an event in order for a rule to be applied to an event.
    ///
    /// A rule with no conditions always matches.
    pub conditions: Vec<PushCondition>,
}

/// Like `PushRule`, but with an additional `pattern` field.
///
/// Only applicable to content rules.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PatternedPushRule {
    /// Actions to determine if and how a notification is delivered for events matching this rule.
    pub actions: Vec<Action>,

    /// Whether this is a default rule, or has been set explicitly.
    pub default: bool,

    /// Whether the push rule is enabled or not.
    pub enabled: bool,

    /// The ID of this rule.
    pub rule_id: String,

    /// The glob-style pattern to match against.
    pub pattern: String,
}

/// A condition that must apply for an associated push rule's action to be taken.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PushCondition {
    /// This is a glob pattern match on a field of the event.
    EventMatch {
        /// The dot-separated field of the event to match.
        key: String,

        /// The glob-style pattern to match against.
        ///
        /// Patterns with no special glob characters should be treated as having asterisks prepended
        /// and appended when testing the condition.
        pattern: String,
    },

    /// This matches unencrypted messages where `content.body` contains the owner's display name in
    /// that room.
    ContainsDisplayName,

    /// This matches the current number of members in the room.
    RoomMemberCount {
        /// A decimal integer optionally prefixed by one of `==`, `<`, `>`, `>=` or `<=`.
        ///
        /// A prefix of `<` matches rooms where the member count is strictly less than the given
        /// number and so forth. If no prefix is present, this parameter defaults to `==`.
        is: String,
    },

    /// This takes into account the current power levels in the room, ensuring the sender of the
    /// event has high enough power to trigger the notification.
    SenderNotificationPermission {
        /// The field in the power level event the user needs a minimum power level for.
        ///
        /// Fields must be specified under the `notifications` property in the power level event's
        /// `content`.
        key: String,
    },
}

#[cfg(test)]
mod tests {
    use matches::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{PushCondition, PushRulesEvent};
    use crate::EventJson;

    #[test]
    fn serialize_event_match_condition() {
        let json_data = json!({
            "key": "content.msgtype",
            "kind": "event_match",
            "pattern": "m.notice"
        });
        assert_eq!(
            to_json_value(&PushCondition::EventMatch {
                key: "content.msgtype".to_string(),
                pattern: "m.notice".to_string(),
            })
            .unwrap(),
            json_data
        );
    }

    #[test]
    fn serialize_contains_display_name_condition() {
        assert_eq!(
            to_json_value(&PushCondition::ContainsDisplayName).unwrap(),
            json!({ "kind": "contains_display_name" })
        );
    }

    #[test]
    fn serialize_room_member_count_condition() {
        let json_data = json!({
            "is": "2",
            "kind": "room_member_count"
        });
        assert_eq!(
            to_json_value(&PushCondition::RoomMemberCount {
                is: "2".to_string(),
            })
            .unwrap(),
            json_data
        );
    }

    #[test]
    fn serialize_sender_notification_permission_condition() {
        let json_data = json!({
            "key": "room",
            "kind": "sender_notification_permission"
        });
        assert_eq!(
            json_data,
            to_json_value(&PushCondition::SenderNotificationPermission {
                key: "room".to_string(),
            })
            .unwrap()
        );
    }

    #[test]
    fn deserialize_event_match_condition() {
        let json_data = json!({
            "key": "content.msgtype",
            "kind": "event_match",
            "pattern": "m.notice"
        });
        assert_matches!(
            from_json_value::<PushCondition>(json_data).unwrap(),
            PushCondition::EventMatch { key, pattern }
            if key == "content.msgtype" && pattern == "m.notice"
        );
    }

    #[test]
    fn deserialize_contains_display_name_condition() {
        assert_matches!(
            from_json_value::<PushCondition>(json!({ "kind": "contains_display_name" })).unwrap(),
            PushCondition::ContainsDisplayName
        );
    }

    #[test]
    fn deserialize_room_member_count_condition() {
        let json_data = json!({
            "is": "2",
            "kind": "room_member_count"
        });
        assert_matches!(
            from_json_value::<PushCondition>(json_data).unwrap(),
            PushCondition::RoomMemberCount { is }
            if is == "2"
        );
    }

    #[test]
    fn deserialize_sender_notification_permission_condition() {
        let json_data = json!({
            "key": "room",
            "kind": "sender_notification_permission"
        });
        assert_matches!(
            from_json_value::<PushCondition>(json_data).unwrap(),
            PushCondition::SenderNotificationPermission {
                key
            } if key == "room"
        );
    }

    #[test]
    fn sanity_check() {
        // This is a full example of a push rules event from the specification.
        let json_data = json!({
            "content": {
                "global": {
                    "content": [
                        {
                            "actions": [
                                "notify",
                                {
                                    "set_tweak": "sound",
                                    "value": "default"
                                },
                                {
                                    "set_tweak": "highlight"
                                }
                            ],
                            "default": true,
                            "enabled": true,
                            "pattern": "alice",
                            "rule_id": ".m.rule.contains_user_name"
                        }
                    ],
                    "override": [
                        {
                            "actions": [
                                "dont_notify"
                            ],
                            "conditions": [],
                            "default": true,
                            "enabled": false,
                            "rule_id": ".m.rule.master"
                        },
                        {
                            "actions": [
                                "dont_notify"
                            ],
                            "conditions": [
                                {
                                    "key": "content.msgtype",
                                    "kind": "event_match",
                                    "pattern": "m.notice"
                                }
                            ],
                            "default": true,
                            "enabled": true,
                            "rule_id": ".m.rule.suppress_notices"
                        }
                    ],
                    "room": [],
                    "sender": [],
                    "underride": [
                        {
                            "actions": [
                                "notify",
                                {
                                    "set_tweak": "sound",
                                    "value": "ring"
                                },
                                {
                                    "set_tweak": "highlight",
                                    "value": false
                                }
                            ],
                            "conditions": [
                                {
                                    "key": "type",
                                    "kind": "event_match",
                                    "pattern": "m.call.invite"
                                }
                            ],
                            "default": true,
                            "enabled": true,
                            "rule_id": ".m.rule.call"
                        },
                        {
                            "actions": [
                                "notify",
                                {
                                    "set_tweak": "sound",
                                    "value": "default"
                                },
                                {
                                    "set_tweak": "highlight"
                                }
                            ],
                            "conditions": [
                                {
                                    "kind": "contains_display_name"
                                }
                            ],
                            "default": true,
                            "enabled": true,
                            "rule_id": ".m.rule.contains_display_name"
                        },
                        {
                            "actions": [
                                "notify",
                                {
                                    "set_tweak": "sound",
                                    "value": "default"
                                },
                                {
                                    "set_tweak": "highlight",
                                    "value": false
                                }
                            ],
                            "conditions": [
                                {
                                    "is": "2",
                                    "kind": "room_member_count"
                                }
                            ],
                            "default": true,
                            "enabled": true,
                            "rule_id": ".m.rule.room_one_to_one"
                        },
                        {
                            "actions": [
                                "notify",
                                {
                                    "set_tweak": "sound",
                                    "value": "default"
                                },
                                {
                                    "set_tweak": "highlight",
                                    "value": false
                                }
                            ],
                            "conditions": [
                                {
                                    "key": "type",
                                    "kind": "event_match",
                                    "pattern": "m.room.member"
                                },
                                {
                                    "key": "content.membership",
                                    "kind": "event_match",
                                    "pattern": "invite"
                                },
                                {
                                    "key": "state_key",
                                    "kind": "event_match",
                                    "pattern": "@alice:example.com"
                                }
                            ],
                            "default": true,
                            "enabled": true,
                            "rule_id": ".m.rule.invite_for_me"
                        },
                        {
                            "actions": [
                                "notify",
                                {
                                    "set_tweak": "highlight",
                                    "value": false
                                }
                            ],
                            "conditions": [
                                {
                                    "key": "type",
                                    "kind": "event_match",
                                    "pattern": "m.room.member"
                                }
                            ],
                            "default": true,
                            "enabled": true,
                            "rule_id": ".m.rule.member_event"
                        },
                        {
                            "actions": [
                                "notify",
                                {
                                    "set_tweak": "highlight",
                                    "value": false
                                }
                            ],
                            "conditions": [
                                {
                                    "key": "type",
                                    "kind": "event_match",
                                    "pattern": "m.room.message"
                                }
                            ],
                            "default": true,
                            "enabled": true,
                            "rule_id": ".m.rule.message"
                        }
                    ]
                }
            },
            "type": "m.push_rules"
        });

        let _ = from_json_value::<EventJson<PushRulesEvent>>(json_data)
            .unwrap()
            .deserialize()
            .unwrap();
    }
}
