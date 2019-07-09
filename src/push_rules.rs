//! Types for the the *m.push_rules* event.

use std::{
    fmt::{Formatter, Result as FmtResult},
    str::FromStr,
};

use ruma_events_macros::ruma_event;
use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_json::{from_value, Value};

use super::{default_true, FromStrError};

ruma_event! {
    /// Describes all push rules for a user.
    PushRulesEvent {
        kind: Event,
        event_type: PushRules,
        content: {
            /// The global ruleset.
            pub global: Ruleset,
        },
    }
}

/// A push ruleset scopes a set of rules according to some criteria.
///
/// For example, some rules may only be applied for messages from a particular sender, a particular
/// room, or by default. The push ruleset contains the entire set of scopes and rules.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Ruleset {
    /// These rules configure behaviour for (unencrypted) messages that match certain patterns.
    pub content: Vec<PatternedPushRule>,

    /// These user-configured rules are given the highest priority.
    #[serde(rename = "override")]
    pub override_rules: Vec<ConditionalPushRule>,

    /// These rules change the behaviour of all messages for a given room.
    pub room: Vec<PushRule>,

    /// These rules configure notification behaviour for messages from a specific Matrix user ID.
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
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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

/// An action affects if and how a notification is delivered for a matching event.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    /// This causes each matching event to generate a notification.
    Notify,

    /// This prevents each matching event from generating a notification.
    DontNotify,

    /// This enables notifications for matching events but activates homeserver specific behaviour
    /// to intelligently coalesce multiple events into a single notification.
    ///
    /// Not all homeservers may support this. Those that do not support it should treat it as the
    /// `notify` action.
    Coalesce,

    /// Sets an entry in the `tweaks` dictionary key that is sent in the notification request to the
    /// Push Gateway. This takes the form of a dictionary with a `set_tweak` key whose value is the
    /// name of the tweak to set. It may also have a `value` key which is the value to which it
    /// should be set.
    SetTweak(Tweak),

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to ruma-events.
    #[doc(hidden)]
    __Nonexhaustive,
}

impl FromStr for Action {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let action = match s {
            "notify" => Action::Notify,
            "dont_notify" => Action::DontNotify,
            "coalesce" => Action::Coalesce,
            _ => return Err(FromStrError),
        };

        Ok(action)
    }
}

impl Serialize for Action {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Action::Notify => serializer.serialize_str("notify"),
            Action::DontNotify => serializer.serialize_str("dont_notify"),
            Action::Coalesce => serializer.serialize_str("coalesce"),
            Action::SetTweak(ref tweak) => tweak.serialize(serializer),
            _ => panic!("Attempted to serialize __Nonexhaustive variant."),
        }
    }
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StringOrStruct;

        impl<'de> Visitor<'de> for StringOrStruct {
            type Value = Action;

            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                formatter.write_str("action as string or map")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match FromStr::from_str(value) {
                    Ok(action) => Ok(action),
                    Err(_) => Err(serde::de::Error::custom("not a string action")),
                }
            }

            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                match Tweak::deserialize(serde::de::value::MapAccessDeserializer::new(map)) {
                    Ok(tweak) => Ok(Action::SetTweak(tweak)),
                    Err(_) => Err(serde::de::Error::custom("unknown action")),
                }
            }
        }

        deserializer.deserialize_any(StringOrStruct)
    }
}

/// Values for the `set_tweak` action.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "set_tweak")]
pub enum Tweak {
    /// A string representing the sound to be played when this notification arrives.
    ///
    /// A value of "default" means to play a default sound. A device may choose to alert the user by
    /// some other means if appropriate, eg. vibration.
    #[serde(rename = "sound")]
    Sound {
        /// The sound to be played.
        value: String,
    },

    /// A boolean representing whether or not this message should be highlighted in the UI.
    ///
    /// This will normally take the form of presenting the message in a different color and/or
    /// style. The UI might also be adjusted to draw particular attention to the room in which the
    /// event occurred. If a `highlight` tweak is given with no value, its value is defined to be
    /// `true`. If no highlight tweak is given at all then the value of `highlight` is defined to be
    /// `false`.
    #[serde(rename = "highlight")]
    Highlight {
        /// Whether or not the message should be highlighted.
        #[serde(default = "default_true")]
        value: bool,
    },
}

/// A condition that must apply for an associated push rule's action to be taken.
#[derive(Clone, Debug, PartialEq)]
pub enum PushCondition {
    /// This is a glob pattern match on a field of the event.
    EventMatch(EventMatchCondition),

    /// This matches unencrypted messages where `content.body` contains the owner's display name in
    /// that room.
    ContainsDisplayName,

    /// This matches the current number of members in the room.
    RoomMemberCount(RoomMemberCountCondition),

    /// This takes into account the current power levels in the room, ensuring the sender of the
    /// event has high enough power to trigger the notification.
    SenderNotificationPermission(SenderNotificationPermissionCondition),

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to ruma-events.
    #[doc(hidden)]
    __Nonexhaustive,
}

impl Serialize for PushCondition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            PushCondition::EventMatch(ref condition) => condition.serialize(serializer),
            PushCondition::ContainsDisplayName => {
                let mut state = serializer.serialize_struct("ContainsDisplayNameCondition", 1)?;

                state.serialize_field("kind", "contains_display_name")?;

                state.end()
            }
            PushCondition::RoomMemberCount(ref condition) => condition.serialize(serializer),
            PushCondition::SenderNotificationPermission(ref condition) => {
                condition.serialize(serializer)
            }
            PushCondition::__Nonexhaustive => {
                panic!("__Nonexhaustive enum variant is not intended for use.");
            }
        }
    }
}

impl<'de> Deserialize<'de> for PushCondition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Deserialize::deserialize(deserializer)?;

        let kind_value = match value.get("kind") {
            Some(value) => value.clone(),
            None => return Err(D::Error::missing_field("kind")),
        };

        let kind = match kind_value.as_str() {
            Some(kind) => kind,
            None => return Err(D::Error::custom("field `kind` must be a string")),
        };

        match kind {
            "event_match" => {
                let condition = match from_value::<EventMatchCondition>(value) {
                    Ok(condition) => condition,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(PushCondition::EventMatch(condition))
            }
            "contains_display_name" => Ok(PushCondition::ContainsDisplayName),
            "room_member_count" => {
                let condition = match from_value::<RoomMemberCountCondition>(value) {
                    Ok(condition) => condition,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(PushCondition::RoomMemberCount(condition))
            }
            "sender_notification_permission" => {
                let condition = match from_value::<SenderNotificationPermissionCondition>(value) {
                    Ok(condition) => condition,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(PushCondition::SenderNotificationPermission(condition))
            }
            unknown_kind => {
                return Err(D::Error::custom(&format!(
                    "unknown condition kind `{}`",
                    unknown_kind
                )))
            }
        }
    }
}
/// A push condition that matches a glob pattern match on a field of the event.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct EventMatchCondition {
    /// The dot-separated field of the event to match.
    pub key: String,

    /// The glob-style pattern to match against.
    ///
    /// Patterns with no special glob characters should be treated as having asterisks prepended and
    /// appended when testing the condition.
    pub pattern: String,
}

impl Serialize for EventMatchCondition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("EventMatchCondition", 3)?;

        state.serialize_field("key", &self.key)?;
        state.serialize_field("kind", "event_match")?;
        state.serialize_field("pattern", &self.pattern)?;

        state.end()
    }
}

/// A push condition that matches the current number of members in the room.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct RoomMemberCountCondition {
    /// A decimal integer optionally prefixed by one of `==`, `<`, `>`, `>=` or `<=`.
    ///
    /// A prefix of `<` matches rooms where the member count is strictly less than the given number
    /// and so forth. If no prefix is present, this parameter defaults to `==`.
    pub is: String,
}

impl Serialize for RoomMemberCountCondition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("RoomMemberCountCondition", 2)?;

        state.serialize_field("is", &self.is)?;
        state.serialize_field("kind", "room_member_count")?;

        state.end()
    }
}

/// A push condition that takes into account the current power levels in the room, ensuring the
/// sender of the event has high enough power to trigger the notification.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SenderNotificationPermissionCondition {
    /// The field in the power level event the user needs a minimum power level for.
    ///
    /// Fields must be specified under the `notifications` property in the power level event's
    /// `content`.
    pub key: String,
}

impl Serialize for SenderNotificationPermissionCondition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("SenderNotificationPermissionCondition", 2)?;

        state.serialize_field("key", &self.key)?;
        state.serialize_field("kind", "sender_notification_permission")?;

        state.end()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_str, to_string};

    use super::{
        Action, EventMatchCondition, PushCondition, PushRulesEvent, RoomMemberCountCondition,
        SenderNotificationPermissionCondition, Tweak,
    };

    #[test]
    fn serialize_string_action() {
        assert_eq!(to_string(&Action::Notify).unwrap(), r#""notify""#);
    }

    #[test]
    fn serialize_tweak_sound_action() {
        assert_eq!(
            to_string(&Action::SetTweak(Tweak::Sound {
                value: "default".to_string()
            }))
            .unwrap(),
            r#"{"set_tweak":"sound","value":"default"}"#
        );
    }

    #[test]
    fn serialize_tweak_highlight_action() {
        assert_eq!(
            to_string(&Action::SetTweak(Tweak::Highlight { value: true })).unwrap(),
            r#"{"set_tweak":"highlight","value":true}"#
        );
    }

    #[test]
    fn deserialize_string_action() {
        assert_eq!(from_str::<Action>(r#""notify""#).unwrap(), Action::Notify);
    }

    #[test]
    fn deserialize_tweak_sound_action() {
        assert_eq!(
            from_str::<Action>(r#"{"set_tweak":"sound","value":"default"}"#).unwrap(),
            Action::SetTweak(Tweak::Sound {
                value: "default".to_string()
            })
        );
    }

    #[test]
    fn deserialize_tweak_highlight_action() {
        assert_eq!(
            from_str::<Action>(r#"{"set_tweak":"highlight","value":true}"#).unwrap(),
            Action::SetTweak(Tweak::Highlight { value: true })
        );
    }

    #[test]
    fn deserialize_tweak_highlight_action_with_default_value() {
        assert_eq!(
            from_str::<Action>(r#"{"set_tweak":"highlight"}"#).unwrap(),
            Action::SetTweak(Tweak::Highlight { value: true })
        );
    }

    #[test]
    fn serialize_event_match_condition() {
        assert_eq!(
            to_string(&PushCondition::EventMatch(EventMatchCondition {
                key: "content.msgtype".to_string(),
                pattern: "m.notice".to_string(),
            }))
            .unwrap(),
            r#"{"key":"content.msgtype","kind":"event_match","pattern":"m.notice"}"#
        );
    }

    #[test]
    fn serialize_contains_display_name_condition() {
        assert_eq!(
            to_string(&PushCondition::ContainsDisplayName).unwrap(),
            r#"{"kind":"contains_display_name"}"#
        );
    }

    #[test]
    fn serialize_room_member_count_condition() {
        assert_eq!(
            to_string(&PushCondition::RoomMemberCount(RoomMemberCountCondition {
                is: "2".to_string(),
            }))
            .unwrap(),
            r#"{"is":"2","kind":"room_member_count"}"#
        );
    }

    #[test]
    fn serialize_sender_notification_permission_condition() {
        assert_eq!(
            r#"{"key":"room","kind":"sender_notification_permission"}"#,
            to_string(&PushCondition::SenderNotificationPermission(
                SenderNotificationPermissionCondition {
                    key: "room".to_string(),
                }
            ))
            .unwrap(),
        );
    }

    #[test]
    fn deserialize_event_match_condition() {
        assert_eq!(
            from_str::<PushCondition>(
                r#"{"key":"content.msgtype","kind":"event_match","pattern":"m.notice"}"#
            )
            .unwrap(),
            PushCondition::EventMatch(EventMatchCondition {
                key: "content.msgtype".to_string(),
                pattern: "m.notice".to_string(),
            })
        );
    }

    #[test]
    fn deserialize_contains_display_name_condition() {
        assert_eq!(
            from_str::<PushCondition>(r#"{"kind":"contains_display_name"}"#).unwrap(),
            PushCondition::ContainsDisplayName,
        );
    }

    #[test]
    fn deserialize_room_member_count_condition() {
        assert_eq!(
            from_str::<PushCondition>(r#"{"is":"2","kind":"room_member_count"}"#).unwrap(),
            PushCondition::RoomMemberCount(RoomMemberCountCondition {
                is: "2".to_string(),
            })
        );
    }

    #[test]
    fn deserialize_sender_notification_permission_condition() {
        assert_eq!(
            from_str::<PushCondition>(r#"{"key":"room","kind":"sender_notification_permission"}"#)
                .unwrap(),
            PushCondition::SenderNotificationPermission(SenderNotificationPermissionCondition {
                key: "room".to_string(),
            })
        );
    }

    #[test]
    fn sanity_check() {
        // This is a full example of a push rules event from the specification.
        let json = r#"{
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
}"#;
        assert!(json.parse::<PushRulesEvent>().is_ok());
    }
}
