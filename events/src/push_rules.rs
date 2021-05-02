//! Types for the *m.push_rules* event.

use ruma_common::push::Ruleset;
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

#[cfg(test)]
mod tests {
    use serde_json::{from_value as from_json_value, json};

    use super::PushRulesEvent;

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

        assert!(from_json_value::<PushRulesEvent>(json_data).is_ok());
    }
}
