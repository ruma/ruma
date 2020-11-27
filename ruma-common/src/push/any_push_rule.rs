use std::{
    convert::TryFrom,
    error::Error,
    fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use super::{
    Action, ConditionalPushRule, ConditionalPushRuleInit, PatternedPushRule, PatternedPushRuleInit,
    PushCondition, SimplePushRule, SimplePushRuleInit,
};

/// Like `SimplePushRule`, but may represent any kind of push rule
/// thanks to `pattern` and `conditions` being optional.
///
/// To create an instance of this type, use one of its `From` implementations.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct AnyPushRule {
    /// The actions to perform when this rule is matched.
    pub actions: Vec<Action>,

    /// Whether this is a default rule, or has been set explicitly.
    pub default: bool,

    /// Whether the push rule is enabled or not.
    pub enabled: bool,

    /// The ID of this rule.
    pub rule_id: String,

    /// The conditions that must hold true for an event in order for a rule to be applied to an
    /// event. A rule with no conditions always matches.
    ///
    /// Only applicable to underride and override rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<Vec<PushCondition>>,

    /// The glob-style pattern to match against. Only applicable to content rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

impl From<SimplePushRule> for AnyPushRule {
    fn from(push_rule: SimplePushRule) -> Self {
        let SimplePushRule { actions, default, enabled, rule_id } = push_rule;
        Self { actions, default, enabled, rule_id, conditions: None, pattern: None }
    }
}

impl From<PatternedPushRule> for AnyPushRule {
    fn from(push_rule: PatternedPushRule) -> Self {
        let PatternedPushRule { actions, default, enabled, rule_id, pattern } = push_rule;
        Self { actions, default, enabled, rule_id, conditions: None, pattern: Some(pattern) }
    }
}

impl From<ConditionalPushRule> for AnyPushRule {
    fn from(push_rule: ConditionalPushRule) -> Self {
        let ConditionalPushRule { actions, default, enabled, rule_id, conditions } = push_rule;
        Self { actions, default, enabled, rule_id, conditions: Some(conditions), pattern: None }
    }
}

impl From<SimplePushRuleInit> for AnyPushRule {
    fn from(init: SimplePushRuleInit) -> Self {
        let SimplePushRuleInit { actions, default, enabled, rule_id } = init;
        Self { actions, default, enabled, rule_id, pattern: None, conditions: None }
    }
}

impl From<ConditionalPushRuleInit> for AnyPushRule {
    fn from(init: ConditionalPushRuleInit) -> Self {
        let ConditionalPushRuleInit { actions, default, enabled, rule_id, conditions } = init;
        Self { actions, default, enabled, rule_id, pattern: None, conditions: Some(conditions) }
    }
}

impl From<PatternedPushRuleInit> for AnyPushRule {
    fn from(init: PatternedPushRuleInit) -> Self {
        let PatternedPushRuleInit { actions, default, enabled, rule_id, pattern } = init;
        Self { actions, default, enabled, rule_id, pattern: Some(pattern), conditions: None }
    }
}

impl From<AnyPushRule> for SimplePushRule {
    fn from(push_rule: AnyPushRule) -> Self {
        let AnyPushRule { actions, default, enabled, rule_id, .. } = push_rule;
        Self { actions, default, enabled, rule_id }
    }
}

/// An error that happens when `AnyPushRule` cannot
/// be converted into `PatternedPushRule`
#[derive(Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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
