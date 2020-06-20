//! Common types for the [push notifications module][push]
//!
//! [push]: https://matrix.org/docs/spec/client_server/r0.6.1#id89

use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use js_int::UInt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::value::RawValue as RawJsonValue;

mod tweak_serde;

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

/// This represents the different actions that should be taken when a rule is matched, and
/// controls how notifications are delivered to the client.
///
/// See https://matrix.org/docs/spec/client_server/r0.6.0#actions for details.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Action {
    /// Causes matching events to generate a notification.
    Notify,

    /// Prevents matching events from generating a notification.
    DontNotify,

    /// Behaves like notify but homeservers may choose to coalesce multiple events
    /// into a single notification.
    Coalesce,

    /// Sets an entry in the 'tweaks' dictionary sent to the push gateway.
    SetTweak(Tweak),
}

/// The `set_tweak` action.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(from = "tweak_serde::Tweak", into = "tweak_serde::Tweak")]
pub enum Tweak {
    /// A string representing the sound to be played when this notification arrives.
    ///
    /// A value of "default" means to play a default sound. A device may choose to alert the user by
    /// some other means if appropriate, eg. vibration.
    Sound(String),

    /// A boolean representing whether or not this message should be highlighted in the UI.
    ///
    /// This will normally take the form of presenting the message in a different color and/or
    /// style. The UI might also be adjusted to draw particular attention to the room in which the
    /// event occurred. If a `highlight` tweak is given with no value, its value is defined to be
    /// `true`. If no highlight tweak is given at all then the value of `highlight` is defined to be
    /// `false`.
    Highlight(#[serde(default = "ruma_serde::default_true")] bool),

    /// A custom tweak
    Custom {
        /// The name of the custom tweak (`set_tweak` field)
        name: String,

        /// The value of the custom tweak
        value: Box<RawJsonValue>,
    },
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};

        struct ActionVisitor;
        impl<'de> Visitor<'de> for ActionVisitor {
            type Value = Action;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                write!(formatter, "a valid action object")
            }

            /// Match a simple action type
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "notify" => Ok(Action::Notify),
                    "dont_notify" => Ok(Action::DontNotify),
                    "coalesce" => Ok(Action::Coalesce),
                    s => Err(E::unknown_variant(&s, &["notify", "dont_notify", "coalesce"])),
                }
            }

            /// Match the more complex set_tweaks action object as a key-value map
            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                Tweak::deserialize(serde::de::value::MapAccessDeserializer::new(map))
                    .map(Action::SetTweak)
            }
        }

        deserializer.deserialize_any(ActionVisitor)
    }
}

impl Serialize for Action {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Action::Notify => serializer.serialize_unit_variant("Action", 0, "notify"),
            Action::DontNotify => serializer.serialize_unit_variant("Action", 1, "dont_notify"),
            Action::Coalesce => serializer.serialize_unit_variant("Action", 2, "coalesce"),
            Action::SetTweak(kind) => kind.serialize(serializer),
        }
    }
}

/// Optional prefix used by `RoomMemberCountIs`
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RoomMemberCountPrefix {
    /// No prefix
    None,
    /// Equals
    Eq,
    /// Less than
    Lt,
    /// Greater than
    Gt,
    /// Greater or equal
    Ge,
    /// Less or equal
    Le,
}

impl Default for RoomMemberCountPrefix {
    fn default() -> Self {
        RoomMemberCountPrefix::None
    }
}

/// A decimal integer optionally prefixed by one of `==`, `<`, `>`, `>=` or `<=`.
///
/// A prefix of `<` matches rooms where the member count is strictly less than the given
/// number and so forth. If no prefix is present, this parameter defaults to `==`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoomMemberCountIs {
    prefix: RoomMemberCountPrefix,
    count: UInt,
}

impl<T> From<T> for RoomMemberCountIs
where
    T: Into<UInt>,
{
    fn from(x: T) -> Self {
        RoomMemberCountIs { prefix: RoomMemberCountPrefix::default(), count: x.into() }
    }
}

impl Display for RoomMemberCountIs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use RoomMemberCountPrefix::*;

        let prefix = match self.prefix {
            None => "",
            Eq => "==",
            Lt => "<",
            Gt => ">",
            Ge => ">=",
            Le => "<=",
        };

        write!(f, "{}{}", prefix, self.count)
    }
}

impl Serialize for RoomMemberCountIs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.to_string();
        s.serialize(serializer)
    }
}

impl FromStr for RoomMemberCountIs {
    type Err = js_int::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use RoomMemberCountPrefix::*;

        let (prefix, count_str) = match s {
            s if s.starts_with("<=") => (Le, &s[2..]),
            s if s.starts_with('<') => (Lt, &s[1..]),
            s if s.starts_with(">=") => (Ge, &s[2..]),
            s if s.starts_with('>') => (Gt, &s[1..]),
            s if s.starts_with("==") => (Eq, &s[2..]),
            s => (None, s),
        };

        Ok(RoomMemberCountIs { prefix, count: UInt::from_str(count_str)? })
    }
}

impl<'de> Deserialize<'de> for RoomMemberCountIs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// A condition that must apply for an associated push rule's action to be taken.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
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
        /// The condition on the current number of members in the room.
        is: RoomMemberCountIs,
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

    use super::{Action, PushCondition, RoomMemberCountIs, Tweak};

    #[test]
    fn serialize_string_action() {
        assert_eq!(to_json_value(&Action::Notify).unwrap(), json!("notify"));
    }

    #[test]
    fn serialize_tweak_sound_action() {
        assert_eq!(
            to_json_value(&Action::SetTweak(Tweak::Sound("default".into()))).unwrap(),
            json!({ "set_tweak": "sound", "value": "default" })
        );
    }

    #[test]
    fn serialize_tweak_highlight_action() {
        assert_eq!(
            to_json_value(&Action::SetTweak(Tweak::Highlight(true))).unwrap(),
            json!({ "set_tweak": "highlight" })
        );

        assert_eq!(
            to_json_value(&Action::SetTweak(Tweak::Highlight(false))).unwrap(),
            json!({ "set_tweak": "highlight", "value": false })
        );
    }

    #[test]
    fn deserialize_string_action() {
        assert_matches!(from_json_value::<Action>(json!("notify")).unwrap(), Action::Notify);
    }

    #[test]
    fn deserialize_tweak_sound_action() {
        let json_data = json!({
            "set_tweak": "sound",
            "value": "default"
        });
        assert_matches!(
            &from_json_value::<Action>(json_data).unwrap(),
            Action::SetTweak(Tweak::Sound(value)) if value == "default"
        );
    }

    #[test]
    fn deserialize_tweak_highlight_action() {
        let json_data = json!({
            "set_tweak": "highlight",
            "value": true
        });
        assert_matches!(
            from_json_value::<Action>(json_data).unwrap(),
            Action::SetTweak(Tweak::Highlight(true))
        );
    }

    #[test]
    fn deserialize_tweak_highlight_action_with_default_value() {
        assert_matches!(
            from_json_value::<Action>(json!({ "set_tweak": "highlight" })).unwrap(),
            Action::SetTweak(Tweak::Highlight(true))
        );
    }

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
            to_json_value(&PushCondition::RoomMemberCount { is: RoomMemberCountIs::from(2u32) })
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
            to_json_value(&PushCondition::SenderNotificationPermission { key: "room".to_string() })
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
            if is == RoomMemberCountIs::from(2u32)
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
}
