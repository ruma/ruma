//! Types for the the *m.push_rules* event.

use std::{
    fmt::{Formatter, Result as FmtResult},
    str::FromStr,
};

use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

use super::{default_true, FromStrError};

event! {
    /// Describes all push rules for a user.
    pub struct PushRulesEvent(PushRulesEventContent) {}
}

/// The payload of an *m.push_rules* event.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PushRulesEventContent {
    /// The global ruleset.
    pub global: Ruleset,
}

/// A push ruleset scopes a set of rules according to some criteria.
///
/// For example, some rules may only be applied for messages from a particular sender, a particular
/// room, or by default. The push ruleset contains the entire set of scopes and rules.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Ruleset {
    /// These rules configure behaviour for (unencrypted) messages that match certain patterns.
    pub content: Vec<PushRule>,

    /// These user-configured rules are given the highest priority.
    #[serde(rename = "override")]
    pub override_rules: Vec<PushRule>,

    /// These rules change the behaviour of all messages for a given room.
    pub room: Vec<PushRule>,

    /// These rules configure notification behaviour for messages from a specific Matrix user ID.
    pub sender: Vec<PushRule>,

    /// These rules are identical to override rules, but have a lower priority than `content`,
    /// `room` and `sender` rules.
    pub underride: Vec<PushRule>,
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

    /// The conditions that must hold true for an event in order for a rule to be applied to an event.
    ///
    /// A rule with no conditions always matches.
    ///
    /// Only applicable to underride and override rules.
    pub conditions: Option<Vec<PushCondition>>,

    /// The glob-style pattern to match against.
    ///
    /// Only applicable to content rules.
    pub pattern: Option<String>,
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
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PushCondition {
    /// The kind of condition to apply.
    pub kind: PushConditionKind,

    /// Required for `event_match` conditions. The dot-separated field of the event to match.
    ///
    /// Required for `sender_notification_permission` conditions. The field in the power level event
    /// the user needs a minimum power level for. Fields must be specified under the `notifications`
    /// property in the power level event's `content`.
    pub key: Option<String>,

    /// Required for `event_match` conditions. The glob-style pattern to match against.
    ///
    /// Patterns with no special glob characters should be treated as having asterisks prepended and
    /// appended when testing the condition.
    pub pattern: Option<String>,

    /// Required for `room_member_count` conditions. A decimal integer optionally prefixed by one of
    /// `==`, `<`, `>`, `>=` or `<=`.
    ///
    /// A prefix of `<` matches rooms where the member count is strictly less than the given number
    /// and so forth. If no prefix is present, this parameter defaults to `==`.
    pub is: Option<String>,
}

/// A kind of push rule condition.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum PushConditionKind {
    /// This is a glob pattern match on a field of the event.
    #[serde(rename = "event_match")]
    EventMatch,

    /// This matches unencrypted messages where `content.body` contains the owner's display name in
    /// that room.
    #[serde(rename = "contains_display_name")]
    ContainsDisplayName,

    /// This matches the current number of members in the room.
    #[serde(rename = "room_member_count")]
    RoomMemberCount,

    /// This takes into account the current power levels in the room, ensuring the sender of the
    /// event has high enough power to trigger the notification.
    #[serde(rename = "sender_notification_permission")]
    SenderNotificationPermission,
}

#[cfg(test)]
mod tests {
    use serde_json::{from_str, to_string};

    use super::{Action, Tweak};

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
}
