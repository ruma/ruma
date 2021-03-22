//! Common types for the [push notifications module][push]
//!
//! [push]: https://matrix.org/docs/spec/client_server/r0.6.1#id89
//!
//! ## Understanding the types of this module
//!
//! Push rules are grouped in `RuleSet`s, and are grouped in five kinds (for
//! more details about the different kind of rules, see the `Ruleset` documentation,
//! or the specification). These five kinds are, by order of priority:
//!
//! - override rules
//! - content rules
//! - room rules
//! - sender rules
//! - underride rules

use std::hash::{Hash, Hasher};

use indexmap::{
    set::{IndexSet, IntoIter as IndexSetIter},
    Equivalent,
};
use ruma_serde::StringEnum;
use serde::{Deserialize, Serialize};

mod action;
mod condition;
mod predefined;

pub use self::{
    action::{Action, Tweak},
    condition::{ComparisonOperator, PushCondition, RoomMemberCountIs},
};

/// A push ruleset scopes a set of rules according to some criteria.
///
/// For example, some rules may only be applied for messages from a particular sender, a particular
/// room, or by default. The push ruleset contains the entire set of scopes and rules.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Ruleset {
    /// These rules configure behavior for (unencrypted) messages that match certain patterns.
    pub content: IndexSet<PatternedPushRule>,

    /// These user-configured rules are given the highest priority.
    ///
    /// This field is named `override_` instead of `override` because the latter is a reserved
    /// keyword in Rust.
    #[serde(rename = "override")]
    pub override_: IndexSet<ConditionalPushRule>,

    /// These rules change the behavior of all messages for a given room.
    pub room: IndexSet<SimplePushRule>,

    /// These rules configure notification behavior for messages from a specific Matrix user ID.
    pub sender: IndexSet<SimplePushRule>,

    /// These rules are identical to override rules, but have a lower priority than `content`,
    /// `room` and `sender` rules.
    pub underride: IndexSet<ConditionalPushRule>,
}

impl Ruleset {
    /// Creates an empty `Ruleset`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Adds a rule to the rule set.
    ///
    /// Returns `true` if the new rule was correctly added, and `false`
    /// if a rule with the same `rule_id` is already present for this kind
    /// of rule.
    pub fn add(&mut self, rule: AnyPushRule) -> bool {
        match rule {
            AnyPushRule::Override(r) => self.override_.insert(r),
            AnyPushRule::Underride(r) => self.underride.insert(r),
            AnyPushRule::Content(r) => self.content.insert(r),
            AnyPushRule::Room(r) => self.room.insert(r),
            AnyPushRule::Sender(r) => self.sender.insert(r),
        }
    }
}

/// Iterator type for `Ruleset`
#[derive(Debug)]
pub struct RulesetIter {
    content: IndexSetIter<PatternedPushRule>,
    override_: IndexSetIter<ConditionalPushRule>,
    room: IndexSetIter<SimplePushRule>,
    sender: IndexSetIter<SimplePushRule>,
    underride: IndexSetIter<ConditionalPushRule>,
}

impl Iterator for RulesetIter {
    type Item = AnyPushRule;

    fn next(&mut self) -> Option<Self::Item> {
        self.override_
            .next()
            .map(AnyPushRule::Override)
            .or_else(|| self.content.next().map(AnyPushRule::Content))
            .or_else(|| self.room.next().map(AnyPushRule::Room))
            .or_else(|| self.sender.next().map(AnyPushRule::Sender))
            .or_else(|| self.underride.next().map(AnyPushRule::Underride))
    }
}

impl IntoIterator for Ruleset {
    type Item = AnyPushRule;
    type IntoIter = RulesetIter;

    fn into_iter(self) -> Self::IntoIter {
        RulesetIter {
            content: self.content.into_iter(),
            override_: self.override_.into_iter(),
            room: self.room.into_iter(),
            sender: self.sender.into_iter(),
            underride: self.underride.into_iter(),
        }
    }
}

/// The kinds of push rules that are available.
#[derive(Clone, Debug)]
pub enum AnyPushRule {
    /// Rules that override all other kinds.
    Override(ConditionalPushRule),

    /// Content-specific rules.
    Content(PatternedPushRule),

    /// Room-specific rules.
    Room(SimplePushRule),

    /// Sender-specific rules.
    Sender(SimplePushRule),

    /// Lowest priority rules.
    Underride(ConditionalPushRule),
}

impl AnyPushRule {
    /// The `rule_id` of the push rule
    pub fn rule_id(&self) -> String {
        match self {
            Self::Override(rule) => rule.rule_id.clone(),
            Self::Underride(rule) => rule.rule_id.clone(),
            Self::Content(rule) => rule.rule_id.clone(),
            Self::Room(rule) => rule.rule_id.clone(),
            Self::Sender(rule) => rule.rule_id.clone(),
        }
    }
}

impl Extend<AnyPushRule> for Ruleset {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = AnyPushRule>,
    {
        for rule in iter {
            self.add(rule);
        }
    }
}

/// A push rule is a single rule that states under what conditions an event should be passed onto a
/// push gateway and how the notification should be presented.
///
/// These rules are stored on the user's homeserver. They are manually configured by the user, who
/// can create and view them via the Client/Server API.
///
/// To create an instance of this type, first create a `SimplePushRuleInit` and convert it via
/// `SimplePushRule::from` / `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SimplePushRule {
    /// Actions to determine if and how a notification is delivered for events matching this rule.
    pub actions: Vec<Action>,

    /// Whether this is a default rule, or has been set explicitly.
    pub default: bool,

    /// Whether the push rule is enabled or not.
    pub enabled: bool,

    /// The ID of this rule.
    pub rule_id: String,
}

/// Initial set of fields of `SimplePushRule`.
///
/// This struct will not be updated even if additional fields are added to `SimplePushRule` in a new
/// (non-breaking) release of the Matrix specification.
#[derive(Debug)]
pub struct SimplePushRuleInit {
    /// Actions to determine if and how a notification is delivered for events matching this rule.
    pub actions: Vec<Action>,

    /// Whether this is a default rule, or has been set explicitly.
    pub default: bool,

    /// Whether the push rule is enabled or not.
    pub enabled: bool,

    /// The ID of this rule.
    pub rule_id: String,
}

impl From<SimplePushRuleInit> for SimplePushRule {
    fn from(init: SimplePushRuleInit) -> Self {
        let SimplePushRuleInit { actions, default, enabled, rule_id } = init;
        Self { actions, default, enabled, rule_id }
    }
}

// The following trait are needed to be able to make
// an IndexSet of the type

impl Hash for SimplePushRule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rule_id.hash(state);
    }
}

impl PartialEq for SimplePushRule {
    fn eq(&self, other: &Self) -> bool {
        self.rule_id == other.rule_id
    }
}

impl Eq for SimplePushRule {}

impl Equivalent<SimplePushRule> for str {
    fn equivalent(&self, key: &SimplePushRule) -> bool {
        self == key.rule_id
    }
}

/// Like `SimplePushRule`, but with an additional `conditions` field.
///
/// Only applicable to underride and override rules.
///
/// To create an instance of this type, first create a `ConditionalPushRuleInit` and convert it via
/// `ConditionalPushRule::from` / `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ConditionalPushRule {
    /// Actions to determine if and how a notification is delivered for events matching this rule.
    pub actions: Vec<Action>,

    /// Whether this is a default rule, or has been set explicitly.
    pub default: bool,

    /// Whether the push rule is enabled or not.
    pub enabled: bool,

    /// The ID of this rule.
    pub rule_id: String,

    /// The conditions that must hold true for an event in order for a rule to be applied to an
    /// event.
    ///
    /// A rule with no conditions always matches.
    pub conditions: Vec<PushCondition>,
}

/// Initial set of fields of `ConditionalPushRule`.
///
/// This struct will not be updated even if additional fields are added to `ConditionalPushRule` in
/// a new (non-breaking) release of the Matrix specification.
#[derive(Debug)]
pub struct ConditionalPushRuleInit {
    /// Actions to determine if and how a notification is delivered for events matching this rule.
    pub actions: Vec<Action>,

    /// Whether this is a default rule, or has been set explicitly.
    pub default: bool,

    /// Whether the push rule is enabled or not.
    pub enabled: bool,

    /// The ID of this rule.
    pub rule_id: String,

    /// The conditions that must hold true for an event in order for a rule to be applied to an
    /// event.
    ///
    /// A rule with no conditions always matches.
    pub conditions: Vec<PushCondition>,
}

impl From<ConditionalPushRuleInit> for ConditionalPushRule {
    fn from(init: ConditionalPushRuleInit) -> Self {
        let ConditionalPushRuleInit { actions, default, enabled, rule_id, conditions } = init;
        Self { actions, default, enabled, rule_id, conditions }
    }
}

// The following trait are needed to be able to make
// an IndexSet of the type

impl Hash for ConditionalPushRule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rule_id.hash(state);
    }
}

impl PartialEq for ConditionalPushRule {
    fn eq(&self, other: &Self) -> bool {
        self.rule_id == other.rule_id
    }
}

impl Eq for ConditionalPushRule {}

impl Equivalent<ConditionalPushRule> for str {
    fn equivalent(&self, key: &ConditionalPushRule) -> bool {
        self == key.rule_id
    }
}

/// Like `SimplePushRule`, but with an additional `pattern` field.
///
/// Only applicable to content rules.
///
/// To create an instance of this type, first create a `PatternedPushRuleInit` and convert it via
/// `PatternedPushRule::from` / `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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

/// Initial set of fields of `PatterenedPushRule`.
///
/// This struct will not be updated even if additional fields are added to `PatterenedPushRule` in a
/// new (non-breaking) release of the Matrix specification.
#[derive(Debug)]
pub struct PatternedPushRuleInit {
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

impl From<PatternedPushRuleInit> for PatternedPushRule {
    fn from(init: PatternedPushRuleInit) -> Self {
        let PatternedPushRuleInit { actions, default, enabled, rule_id, pattern } = init;
        Self { actions, default, enabled, rule_id, pattern }
    }
}

// The following trait are needed to be able to make
// an IndexSet of the type

impl Hash for PatternedPushRule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rule_id.hash(state);
    }
}

impl PartialEq for PatternedPushRule {
    fn eq(&self, other: &Self) -> bool {
        self.rule_id == other.rule_id
    }
}

impl Eq for PatternedPushRule {}

impl Equivalent<PatternedPushRule> for str {
    fn equivalent(&self, key: &PatternedPushRule) -> bool {
        self == key.rule_id
    }
}

/// Information for the pusher implementation itself.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PusherData {
    /// Required if the pusher's kind is http. The URL to use to send notifications to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// The format to use when sending notifications to the Push Gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<PushFormat>,
}

impl PusherData {
    /// Creates an empty `PusherData`.
    pub fn new() -> Self {
        Default::default()
    }
}

/// A special format that the homeserver should use when sending notifications to a Push Gateway.
/// Currently, only "event_id_only" is supported as of [Push Gateway API r0.1.1][spec].
///
/// [spec]: https://matrix.org/docs/spec/push_gateway/r0.1.1#homeserver-behaviour
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
pub enum PushFormat {
    /// Require the homeserver to only send a reduced set of fields in the push.
    EventIdOnly,

    #[doc(hidden)]
    _Custom(String),
}

#[cfg(test)]
mod tests {
    use js_int::uint;
    use matches::assert_matches;
    use serde_json::{
        from_value as from_json_value, json, to_value as to_json_value,
        value::RawValue as RawJsonValue, Value as JsonValue,
    };

    use super::{
        action::{Action, Tweak},
        condition::{PushCondition, RoomMemberCountIs},
        AnyPushRule, ConditionalPushRule, PatternedPushRule, Ruleset, SimplePushRule,
    };

    fn example_ruleset() -> Ruleset {
        let mut set = Ruleset::new();

        set.add(AnyPushRule::Override(ConditionalPushRule {
            conditions: vec![PushCondition::EventMatch {
                key: "type".into(),
                pattern: "m.call.invite".into(),
            }],
            actions: vec![Action::Notify, Action::SetTweak(Tweak::Highlight(true))],
            rule_id: ".m.rule.call".into(),
            enabled: true,
            default: true,
        }));

        set
    }

    #[test]
    fn cannot_add_same_rule_id() {
        let mut set = example_ruleset();

        let added = set.add(AnyPushRule::Override(ConditionalPushRule {
            conditions: vec![],
            actions: vec![],
            rule_id: ".m.rule.call".into(),
            enabled: true,
            default: true,
        }));

        assert!(!added);
    }

    #[test]
    fn can_add_same_rule_id_different_kind() {
        let mut set = example_ruleset();

        let added = set.add(AnyPushRule::Underride(ConditionalPushRule {
            conditions: vec![],
            actions: vec![],
            rule_id: ".m.rule.call".into(),
            enabled: true,
            default: true,
        }));

        assert!(added);
    }

    #[test]
    fn get_by_rule_id() {
        let set = example_ruleset();

        let rule = set.override_.get(".m.rule.call");
        assert!(rule.is_some());
        assert_eq!(rule.unwrap().rule_id, ".m.rule.call");

        let rule = set.override_.get(".m.rule.doesntexist");
        assert!(rule.is_none());
    }

    #[test]
    fn iter() {
        let mut set = example_ruleset();

        let added = set.add(AnyPushRule::Override(ConditionalPushRule {
            conditions: vec![PushCondition::EventMatch {
                key: "room_id".into(),
                pattern: "!roomid:matrix.org".into(),
            }],
            actions: vec![Action::DontNotify],
            rule_id: "!roomid:matrix.org".into(),
            enabled: true,
            default: false,
        }));
        assert!(added);

        let added = set.add(AnyPushRule::Override(ConditionalPushRule {
            conditions: vec![],
            actions: vec![],
            rule_id: ".m.rule.suppress_notices".into(),
            enabled: false,
            default: true,
        }));
        assert!(added);

        let mut iter = set.into_iter();

        let rule_opt = iter.next();
        assert!(rule_opt.is_some());
        assert_matches!(
            rule_opt.unwrap(),
            AnyPushRule::Override(ConditionalPushRule { rule_id, .. })
            if rule_id == ".m.rule.call"
        );

        let rule_opt = iter.next();
        assert!(rule_opt.is_some());
        assert_matches!(
            rule_opt.unwrap(),
            AnyPushRule::Override(ConditionalPushRule { rule_id, .. })
            if rule_id == "!roomid:matrix.org"
        );

        let rule_opt = iter.next();
        assert!(rule_opt.is_some());
        assert_matches!(
            rule_opt.unwrap(),
            AnyPushRule::Override(ConditionalPushRule { rule_id, .. })
            if rule_id == ".m.rule.suppress_notices"
        );

        assert!(iter.next().is_none());
    }

    #[test]
    fn serialize_conditional_push_rule() {
        let rule = ConditionalPushRule {
            actions: vec![Action::Notify, Action::SetTweak(Tweak::Highlight(true))],
            default: true,
            enabled: true,
            rule_id: ".m.rule.call".into(),
            conditions: vec![
                PushCondition::EventMatch { key: "type".into(), pattern: "m.call.invite".into() },
                PushCondition::ContainsDisplayName,
                PushCondition::RoomMemberCount { is: RoomMemberCountIs::gt(uint!(2)) },
                PushCondition::SenderNotificationPermission { key: "room".into() },
            ],
        };

        let rule_value: JsonValue = to_json_value(rule).unwrap();
        assert_eq!(
            rule_value,
            json!({
                "conditions": [
                    {
                        "kind": "event_match",
                        "key": "type",
                        "pattern": "m.call.invite"
                    },
                    {
                        "kind": "contains_display_name"
                    },
                    {
                        "kind": "room_member_count",
                        "is": ">2"
                    },
                    {
                        "kind": "sender_notification_permission",
                        "key": "room"
                    }
                ],
                "actions": [
                    "notify",
                    {
                        "set_tweak": "highlight"
                    }
                ],
                "rule_id": ".m.rule.call",
                "default": true,
                "enabled": true
            })
        );
    }

    #[test]
    fn serialize_simple_push_rule() {
        let rule = SimplePushRule {
            actions: vec![Action::DontNotify],
            default: false,
            enabled: false,
            rule_id: "!roomid:server.name".into(),
        };

        let rule_value: JsonValue = to_json_value(rule).unwrap();
        assert_eq!(
            rule_value,
            json!({
                "actions": [
                    "dont_notify"
                ],
                "rule_id": "!roomid:server.name",
                "default": false,
                "enabled": false
            })
        );
    }

    #[test]
    fn serialize_patterned_push_rule() {
        let rule = PatternedPushRule {
            actions: vec![
                Action::Notify,
                Action::SetTweak(Tweak::Sound("default".into())),
                Action::SetTweak(Tweak::Custom {
                    name: "dance".into(),
                    value: RawJsonValue::from_string("true".into()).unwrap(),
                }),
            ],
            default: true,
            enabled: true,
            pattern: "user_id".into(),
            rule_id: ".m.rule.contains_user_name".into(),
        };

        let rule_value: JsonValue = to_json_value(rule).unwrap();
        assert_eq!(
            rule_value,
            json!({
                "actions": [
                    "notify",
                    {
                        "set_tweak": "sound",
                        "value": "default"
                    },
                    {
                        "set_tweak": "dance",
                        "value": true
                    }
                ],
                "pattern": "user_id",
                "rule_id": ".m.rule.contains_user_name",
                "default": true,
                "enabled": true
            })
        );
    }

    #[test]
    fn serialize_ruleset() {
        let mut set = example_ruleset();

        set.add(AnyPushRule::Override(ConditionalPushRule {
            conditions: vec![
                PushCondition::RoomMemberCount { is: RoomMemberCountIs::from(uint!(2)) },
                PushCondition::EventMatch { key: "type".into(), pattern: "m.room.message".into() },
            ],
            actions: vec![
                Action::Notify,
                Action::SetTweak(Tweak::Sound("default".into())),
                Action::SetTweak(Tweak::Highlight(false)),
            ],
            rule_id: ".m.rule.room_one_to_one".into(),
            enabled: true,
            default: true,
        }));
        set.add(AnyPushRule::Content(PatternedPushRule {
            actions: vec![
                Action::Notify,
                Action::SetTweak(Tweak::Sound("default".into())),
                Action::SetTweak(Tweak::Highlight(true)),
            ],
            rule_id: ".m.rule.contains_user_name".into(),
            pattern: "user_id".into(),
            enabled: true,
            default: true,
        }));

        let set_value: JsonValue = to_json_value(set).unwrap();
        assert_eq!(
            set_value,
            json!({
                "override": [
                    {
                        "actions": [
                            "notify",
                            {
                                "set_tweak": "highlight",
                            },
                        ],
                        "conditions": [
                            {
                                "kind": "event_match",
                                "key": "type",
                                "pattern": "m.call.invite"
                            },
                        ],
                        "rule_id": ".m.rule.call",
                        "default": true,
                        "enabled": true,
                    },
                    {
                        "conditions": [
                            {
                                "kind": "room_member_count",
                                "is": "2"
                            },
                            {
                                "kind": "event_match",
                                "key": "type",
                                "pattern": "m.room.message"
                            }
                        ],
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
                        "rule_id": ".m.rule.room_one_to_one",
                        "default": true,
                        "enabled": true
                    },
                ],
                "room": [],
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
                        "pattern": "user_id",
                        "rule_id": ".m.rule.contains_user_name",
                        "default": true,
                        "enabled": true
                    }
                ],
                "sender": [],
                "underride": [],
            })
        );
    }

    #[test]
    fn deserialize_patterned_push_rule() {
        let rule = from_json_value(json!({
            "actions": [
                "notify",
                {
                    "set_tweak": "sound",
                    "value": "default"
                },
                {
                    "set_tweak": "highlight",
                    "value": true
                }
            ],
            "pattern": "user_id",
            "rule_id": ".m.rule.contains_user_name",
            "default": true,
            "enabled": true
        }))
        .unwrap();
        assert_matches!(
            rule,
            PatternedPushRule {
                actions: _,
                default: true,
                enabled: true,
                pattern,
                rule_id,
            }
            if pattern == "user_id" && rule_id == ".m.rule.contains_user_name"
        );

        let mut iter = rule.actions.iter();
        assert_matches!(iter.next(), Some(Action::Notify));
        assert_matches!(iter.next(), Some(Action::SetTweak(Tweak::Sound(sound))) if sound == "default");
        assert_matches!(iter.next(), Some(Action::SetTweak(Tweak::Highlight(true))));
        assert_matches!(iter.next(), None);
    }

    #[test]
    fn deserialize_ruleset() {
        let set: Ruleset = from_json_value(json!({
            "override": [
                {
                    "actions": [],
                    "conditions": [],
                    "rule_id": "!roomid:server.name",
                    "default": false,
                    "enabled": true
                },
                {
                    "actions": [],
                    "conditions": [],
                    "rule_id": ".m.rule.call",
                    "default": true,
                    "enabled": true
                },
            ],
            "underride": [
                {
                    "actions": [],
                    "conditions": [],
                    "rule_id": ".m.rule.room_one_to_one",
                    "default": true,
                    "enabled": true
                },
            ],
            "room": [
                {
                    "actions": [],
                    "rule_id": "!roomid:server.name",
                    "default": false,
                    "enabled": false
                }
            ],
            "sender": [],
            "content": [
                {
                    "actions": [],
                    "pattern": "user_id",
                    "rule_id": ".m.rule.contains_user_name",
                    "default": true,
                    "enabled": true
                },
                {
                    "actions": [],
                    "pattern": "ruma",
                    "rule_id": "ruma",
                    "default": false,
                    "enabled": true
                }
            ]
        }))
        .unwrap();

        let mut iter = set.into_iter();

        let rule_opt = iter.next();
        assert!(rule_opt.is_some());
        assert_matches!(
            rule_opt.unwrap(),
            AnyPushRule::Override(ConditionalPushRule { rule_id, .. })
            if rule_id == "!roomid:server.name"
        );

        let rule_opt = iter.next();
        assert!(rule_opt.is_some());
        assert_matches!(
            rule_opt.unwrap(),
            AnyPushRule::Override(ConditionalPushRule { rule_id, .. })
            if rule_id == ".m.rule.call"
        );

        let rule_opt = iter.next();
        assert!(rule_opt.is_some());
        assert_matches!(
            rule_opt.unwrap(),
            AnyPushRule::Content(PatternedPushRule { rule_id, .. })
            if rule_id == ".m.rule.contains_user_name"
        );

        let rule_opt = iter.next();
        assert!(rule_opt.is_some());
        assert_matches!(
            rule_opt.unwrap(),
            AnyPushRule::Content(PatternedPushRule { rule_id, .. })
            if rule_id == "ruma"
        );

        let rule_opt = iter.next();
        assert!(rule_opt.is_some());
        assert_matches!(
            rule_opt.unwrap(),
            AnyPushRule::Room(SimplePushRule { rule_id, .. })
            if rule_id == "!roomid:server.name"
        );

        let rule_opt = iter.next();
        assert!(rule_opt.is_some());
        assert_matches!(
            rule_opt.unwrap(),
            AnyPushRule::Underride(ConditionalPushRule { rule_id, .. })
            if rule_id == ".m.rule.room_one_to_one"
        );

        assert!(iter.next().is_none());
    }
}
