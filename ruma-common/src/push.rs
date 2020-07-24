//! Common types for the [push notifications module][push]
//!
//! [push]: https://matrix.org/docs/spec/client_server/r0.6.1#id89

use std::{
    convert::TryFrom,
    error::Error,
    fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

pub use self::{
    action::{Action, Tweak},
    condition::{ComparisonOperator, PushCondition, RoomMemberCountIs},
};

mod action;
mod condition;

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

/// Like `PushRule`, but may represent any kind of push rule
/// thanks to `pattern` and `conditions` being optional.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnyPushRule {
    /// The actions to perform when this rule is matched.
    pub actions: Vec<Action>,

    /// Whether this is a default rule, or has been set explicitly.
    pub default: bool,

    /// Whether the push rule is enabled or not.
    pub enabled: bool,

    /// The ID of this rule.
    pub rule_id: String,

    /// The conditions that must hold true for an event in order for a rule to be applied to an event. A rule with no conditions always matches.
    /// Only applicable to underride and override rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<Vec<PushCondition>>,

    /// The glob-style pattern to match against. Only applicable to content rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

impl From<PushRule> for AnyPushRule {
    fn from(push_rule: PushRule) -> Self {
        let PushRule { actions, default, enabled, rule_id } = push_rule;
        AnyPushRule { actions, default, enabled, rule_id, pattern: None, conditions: None }
    }
}

impl From<PatternedPushRule> for AnyPushRule {
    fn from(push_rule: PatternedPushRule) -> Self {
        let PatternedPushRule { actions, default, enabled, rule_id, pattern } = push_rule;
        AnyPushRule { actions, default, enabled, rule_id, pattern: Some(pattern), conditions: None }
    }
}

impl From<ConditionalPushRule> for AnyPushRule {
    fn from(push_rule: ConditionalPushRule) -> Self {
        let ConditionalPushRule { actions, default, enabled, rule_id, conditions } = push_rule;
        AnyPushRule {
            actions,
            default,
            enabled,
            rule_id,
            pattern: None,
            conditions: Some(conditions),
        }
    }
}

impl From<AnyPushRule> for PushRule {
    fn from(push_rule: AnyPushRule) -> Self {
        let AnyPushRule { actions, default, enabled, rule_id, .. } = push_rule;
        PushRule { actions, default, enabled, rule_id }
    }
}

/// An error that happens when `AnyPushRule` cannot
/// be converted into `PatternedPushRule`
#[derive(Debug)]
pub struct MissingPatternError;

impl Display for MissingPatternError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Push rule does not have a pattern.")
    }
}

impl Error for MissingPatternError {}

impl TryFrom<AnyPushRule> for PatternedPushRule {
    type Error = MissingPatternError;

    fn try_from(push_rule: AnyPushRule) -> Result<Self, Self::Error> {
        if let AnyPushRule { actions, default, enabled, rule_id, pattern: Some(pattern), .. } =
            push_rule
        {
            Ok(PatternedPushRule { actions, default, enabled, rule_id, pattern })
        } else {
            Err(MissingPatternError)
        }
    }
}

/// An error that happens when `AnyPushRule` cannot
/// be converted into `ConditionalPushRule`
#[derive(Debug)]
pub struct MissingConditionsError;

impl Display for MissingConditionsError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Push rule has no conditions.")
    }
}

impl Error for MissingConditionsError {}

impl TryFrom<AnyPushRule> for ConditionalPushRule {
    type Error = MissingConditionsError;

    fn try_from(push_rule: AnyPushRule) -> Result<Self, Self::Error> {
        if let AnyPushRule {
            actions,
            default,
            enabled,
            rule_id,
            conditions: Some(conditions),
            ..
        } = push_rule
        {
            Ok(ConditionalPushRule { actions, default, enabled, rule_id, conditions })
        } else {
            Err(MissingConditionsError)
        }
    }
}
