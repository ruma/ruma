//! Endpoints for push notifications.
use std::{error::Error, fmt};

pub use ruma_common::push::RuleKind;
use ruma_common::{
    push::{
        Action, AnyPushRule, AnyPushRuleRef, ConditionalPushRule, ConditionalPushRuleInit,
        HttpPusherData, PatternedPushRule, PatternedPushRuleInit, PushCondition, SimplePushRule,
        SimplePushRuleInit,
    },
    serde::JsonObject,
};
use serde::{Deserialize, Serialize};

pub mod delete_pushrule;
pub mod get_notifications;
pub mod get_pushers;
pub mod get_pushrule;
pub mod get_pushrule_actions;
pub mod get_pushrule_enabled;
pub mod get_pushrules_all;
pub mod get_pushrules_global_scope;
mod pusher_serde;
pub mod set_pusher;
pub mod set_pushrule;
pub mod set_pushrule_actions;
pub mod set_pushrule_enabled;

/// Like `SimplePushRule`, but may represent any kind of push rule thanks to `pattern` and
/// `conditions` being optional.
///
/// To create an instance of this type, use one of its `From` implementations.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PushRule {
    /// The actions to perform when this rule is matched.
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
    /// A rule with no conditions always matches. Only applicable to underride and override rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<Vec<PushCondition>>,

    /// The glob-style pattern to match against.
    ///
    /// Only applicable to content rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

impl<T> From<SimplePushRule<T>> for PushRule
where
    T: Into<String>,
{
    fn from(push_rule: SimplePushRule<T>) -> Self {
        let SimplePushRule { actions, default, enabled, rule_id, .. } = push_rule;
        let rule_id = rule_id.into();
        Self { actions, default, enabled, rule_id, conditions: None, pattern: None }
    }
}

impl From<PatternedPushRule> for PushRule {
    fn from(push_rule: PatternedPushRule) -> Self {
        let PatternedPushRule { actions, default, enabled, rule_id, pattern, .. } = push_rule;
        Self { actions, default, enabled, rule_id, conditions: None, pattern: Some(pattern) }
    }
}

impl From<ConditionalPushRule> for PushRule {
    fn from(push_rule: ConditionalPushRule) -> Self {
        let ConditionalPushRule { actions, default, enabled, rule_id, conditions, .. } = push_rule;
        Self { actions, default, enabled, rule_id, conditions: Some(conditions), pattern: None }
    }
}

impl<T> From<SimplePushRuleInit<T>> for PushRule
where
    T: Into<String>,
{
    fn from(init: SimplePushRuleInit<T>) -> Self {
        let SimplePushRuleInit { actions, default, enabled, rule_id } = init;
        let rule_id = rule_id.into();
        Self { actions, default, enabled, rule_id, pattern: None, conditions: None }
    }
}

impl From<ConditionalPushRuleInit> for PushRule {
    fn from(init: ConditionalPushRuleInit) -> Self {
        let ConditionalPushRuleInit { actions, default, enabled, rule_id, conditions } = init;
        Self { actions, default, enabled, rule_id, pattern: None, conditions: Some(conditions) }
    }
}

impl From<PatternedPushRuleInit> for PushRule {
    fn from(init: PatternedPushRuleInit) -> Self {
        let PatternedPushRuleInit { actions, default, enabled, rule_id, pattern } = init;
        Self { actions, default, enabled, rule_id, pattern: Some(pattern), conditions: None }
    }
}

impl From<AnyPushRule> for PushRule {
    fn from(push_rule: AnyPushRule) -> Self {
        // The catch-all is unreachable if the "unstable-exhaustive-types" feature is enabled.
        #[allow(unreachable_patterns)]
        match push_rule {
            AnyPushRule::Override(r) => r.into(),
            AnyPushRule::Content(r) => r.into(),
            AnyPushRule::Room(r) => r.into(),
            AnyPushRule::Sender(r) => r.into(),
            AnyPushRule::Underride(r) => r.into(),
            _ => unreachable!(),
        }
    }
}

impl<'a> From<AnyPushRuleRef<'a>> for PushRule {
    fn from(push_rule: AnyPushRuleRef<'a>) -> Self {
        push_rule.to_owned().into()
    }
}

impl<T> TryFrom<PushRule> for SimplePushRule<T>
where
    T: TryFrom<String>,
{
    type Error = <T as TryFrom<String>>::Error;

    fn try_from(push_rule: PushRule) -> Result<Self, Self::Error> {
        let PushRule { actions, default, enabled, rule_id, .. } = push_rule;
        let rule_id = T::try_from(rule_id)?;
        Ok(SimplePushRuleInit { actions, default, enabled, rule_id }.into())
    }
}

/// An error that happens when `PushRule` cannot
/// be converted into `PatternedPushRule`
#[derive(Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct MissingPatternError;

impl fmt::Display for MissingPatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Push rule does not have a pattern.")
    }
}

impl Error for MissingPatternError {}

impl TryFrom<PushRule> for PatternedPushRule {
    type Error = MissingPatternError;

    fn try_from(push_rule: PushRule) -> Result<Self, Self::Error> {
        if let PushRule { actions, default, enabled, rule_id, pattern: Some(pattern), .. } =
            push_rule
        {
            Ok(PatternedPushRuleInit { actions, default, enabled, rule_id, pattern }.into())
        } else {
            Err(MissingPatternError)
        }
    }
}

impl From<PushRule> for ConditionalPushRule {
    fn from(push_rule: PushRule) -> Self {
        let PushRule { actions, default, enabled, rule_id, conditions, .. } = push_rule;

        ConditionalPushRuleInit {
            actions,
            default,
            enabled,
            rule_id,
            conditions: conditions.unwrap_or_default(),
        }
        .into()
    }
}

/// Which kind a pusher is, and the information for that kind.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum PusherKind {
    /// A pusher that sends HTTP pokes.
    Http(HttpPusherData),

    /// A pusher that emails the user with unread notifications.
    Email(EmailPusherData),

    #[doc(hidden)]
    _Custom(CustomPusherData),
}

/// Defines a pusher.
///
/// To create an instance of this type, first create a `PusherInit` and convert it via
/// `Pusher::from` / `.into()`.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Pusher {
    /// Identifiers for this pusher.
    #[serde(flatten)]
    pub ids: PusherIds,

    /// The kind of the pusher and the information for that kind.
    #[serde(flatten)]
    pub kind: PusherKind,

    /// A string that will allow the user to identify what application owns this pusher.
    pub app_display_name: String,

    /// A string that will allow the user to identify what device owns this pusher.
    pub device_display_name: String,

    /// Determines which set of device specific rules this pusher executes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_tag: Option<String>,

    /// The preferred language for receiving notifications (e.g. 'en' or 'en-US')
    pub lang: String,
}

/// Initial set of fields of `Pusher`.
///
/// This struct will not be updated even if additional fields are added to `Pusher` in a new
/// (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct PusherInit {
    /// Identifiers for this pusher.
    pub ids: PusherIds,

    /// The kind of the pusher.
    pub kind: PusherKind,

    /// A string that will allow the user to identify what application owns this pusher.
    pub app_display_name: String,

    /// A string that will allow the user to identify what device owns this pusher.
    pub device_display_name: String,

    /// Determines which set of device-specific rules this pusher executes.
    pub profile_tag: Option<String>,

    /// The preferred language for receiving notifications (e.g. 'en' or 'en-US').
    pub lang: String,
}

impl From<PusherInit> for Pusher {
    fn from(init: PusherInit) -> Self {
        let PusherInit { ids, kind, app_display_name, device_display_name, profile_tag, lang } =
            init;
        Self { ids, kind, app_display_name, device_display_name, profile_tag, lang }
    }
}

/// Strings to uniquely identify a `Pusher`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PusherIds {
    /// A unique identifier for the pusher.
    ///
    /// The maximum allowed length is 512 bytes.
    pub pushkey: String,

    /// A reverse-DNS style identifier for the application.
    ///
    /// The maximum allowed length is 64 bytes.
    pub app_id: String,
}

impl PusherIds {
    /// Creates a new `PusherIds` with the given pushkey and application ID.
    pub fn new(pushkey: String, app_id: String) -> Self {
        Self { pushkey, app_id }
    }
}

/// Information for an email pusher.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(transparent, default)]
pub struct EmailPusherData {
    /// Custom data for the pusher.
    pub data: JsonObject,
}

impl EmailPusherData {
    /// Creates a new empty `EmailPusherData`.
    pub fn new() -> Self {
        Self::default()
    }
}

#[doc(hidden)]
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct CustomPusherData {
    kind: String,
    data: JsonObject,
}
