//! Common types for the [push notifications module][push]
//!
//! [push]: https://matrix.org/docs/spec/client_server/r0.6.1#id89
//!
//! ## Understanding the types of this module
//!
//! Push rules are grouped in `RuleSet`s, and are grouped in five kinds (for
//! more details about the different kind of rules, see the `Ruleset` documentation,
//! or the specification). These five kinds are:
//!
//! - content rules
//! - override rules
//! - underride rules
//! - room rules
//! - sender rules
//!
//! Each of these kind of rule has a corresponding type that is
//! just a wrapper around another type:
//!
//! - `SimplePushRule` for room and sender rules
//! - `ConditionalPushRule` for override and underride rules: push rules that may depend on a
//!   condition
//! - `PatternedPushRules` for content rules, that can filter events based on a pattern to trigger
//!   the rule or not
//!
//! Having these wrapper types allows to tell at the type level what kind of rule you are
//! handling, and makes sure the `Ruleset::add` method adds your rule to the correct field
//! of `Ruleset`, and that rules that are not of the same kind are never mixed even if they share
//! the same representation.
//!
//! It is still possible to write code that is generic over a representation by manipulating
//! `SimplePushRule`, `ConditonalPushRule` or `PatternedPushRule` directly, instead of the wrappers.
//!
//! There is also the `AnyPushRule` type that is the most generic form of push rule, with all
//! the possible fields.

use std::collections::btree_set::{BTreeSet, IntoIter as BTreeSetIter};

use ruma_serde::StringEnum;
use serde::{Deserialize, Serialize};

mod action;
mod any_push_rule;
mod condition;
mod predefined;

pub use self::{
    action::{Action, Tweak},
    any_push_rule::{AnyPushRule, MissingConditionsError, MissingPatternError},
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
    pub content: BTreeSet<ContentPushRule>,

    /// These user-configured rules are given the highest priority.
    ///
    /// This field is named `override_` instead of `override` because the latter is a reserved
    /// keyword in Rust.
    #[serde(rename = "override")]
    pub override_: BTreeSet<OverridePushRule>,

    /// These rules change the behavior of all messages for a given room.
    pub room: BTreeSet<RoomPushRule>,

    /// These rules configure notification behavior for messages from a specific Matrix user ID.
    pub sender: BTreeSet<SenderPushRule>,

    /// These rules are identical to override rules, but have a lower priority than `content`,
    /// `room` and `sender` rules.
    pub underride: BTreeSet<UnderridePushRule>,
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
    pub fn add<R: RulesetMember>(&mut self, rule: R) -> bool {
        rule.add_to(self)
    }
}

/// Iterator type for `Ruleset`
#[derive(Debug)]
pub struct RulesetIter {
    content: BTreeSetIter<ContentPushRule>,
    override_: BTreeSetIter<OverridePushRule>,
    room: BTreeSetIter<RoomPushRule>,
    sender: BTreeSetIter<SenderPushRule>,
    underride: BTreeSetIter<UnderridePushRule>,
}

impl Iterator for RulesetIter {
    type Item = AnyPushRule;

    fn next(&mut self) -> Option<Self::Item> {
        self.content
            .next()
            .map(|x| x.0.into())
            .or_else(|| self.override_.next().map(|x| x.0.into()))
            .or_else(|| self.room.next().map(|x| x.0.into()))
            .or_else(|| self.sender.next().map(|x| x.0.into()))
            .or_else(|| self.underride.next().map(|x| x.0.into()))
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

/// A trait for types that can be added in a Ruleset
pub trait RulesetMember: private::Sealed {
    /// Adds a value in the correct field of a Ruleset.
    #[doc(hidden)]
    fn add_to(self, ruleset: &mut Ruleset) -> bool;
}

mod private {
    // See <https://rust-lang.github.io/api-guidelines/future-proofing.html>
    pub trait Sealed {}
    impl Sealed for super::OverridePushRule {}
    impl Sealed for super::UnderridePushRule {}
    impl Sealed for super::ContentPushRule {}
    impl Sealed for super::RoomPushRule {}
    impl Sealed for super::SenderPushRule {}
}

/// Creates a new wrapper type around a PushRule-like type
/// to make it possible to tell what kind of rule it is
/// even if the inner type is the same.
///
/// For instance, override and underride rules are both
/// represented as `ConditionalPushRule`s, so it is impossible
/// to tell if a rule is an override or an underride rule when
/// all you have is a `ConditionalPushRule`. With these wrapper types
/// it becomes possible.
macro_rules! rulekind {
    ($name:ident, $inner:ty, $field:ident) => {
        #[derive(Clone, Debug, Serialize, Deserialize)]
        #[doc = "Wrapper type to disambiguate the kind of the wrapped rule"]
        pub struct $name(pub $inner);

        impl RulesetMember for $name {
            fn add_to(self, ruleset: &mut Ruleset) -> bool {
                ruleset.$field.insert(self)
            }
        }

        impl Extend<$name> for Ruleset {
            fn extend<T: IntoIterator<Item = $name>>(&mut self, iter: T) {
                for rule in iter {
                    rule.add_to(self);
                }
            }
        }

        // The following trait are needed to be able to make
        // a BTreeSet of the new type

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                self.0.rule_id.cmp(&other.0.rule_id)
            }
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.0.rule_id == other.0.rule_id
            }
        }

        impl Eq for $name {}
    };
}

rulekind!(OverridePushRule, ConditionalPushRule, override_);
rulekind!(UnderridePushRule, ConditionalPushRule, underride);
rulekind!(RoomPushRule, SimplePushRule, room);
rulekind!(SenderPushRule, SimplePushRule, sender);
rulekind!(ContentPushRule, PatternedPushRule, content);

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
