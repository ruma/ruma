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
