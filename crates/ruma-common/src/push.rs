//! Common types for the [push notifications module][push].
//!
//! [push]: https://spec.matrix.org/v1.4/client-server-api/#push-notifications
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

use indexmap::{Equivalent, IndexSet};
use serde::{Deserialize, Serialize};
#[cfg(feature = "unstable-unspecified")]
use serde_json::Value as JsonValue;
use thiserror::Error;
use tracing::instrument;

use crate::{
    serde::{Raw, StringEnum},
    OwnedRoomId, OwnedUserId, PrivOwnedStr,
};

mod action;
mod condition;
mod iter;
mod predefined;

#[cfg(feature = "unstable-msc3932")]
pub use self::condition::RoomVersionFeature;
pub use self::{
    action::{Action, Tweak},
    condition::{
        ComparisonOperator, FlattenedJson, PushCondition, PushConditionRoomCtx, RoomMemberCountIs,
        _CustomPushCondition,
    },
    iter::{AnyPushRule, AnyPushRuleRef, RulesetIntoIter, RulesetIter},
    predefined::{
        PredefinedContentRuleId, PredefinedOverrideRuleId, PredefinedRuleId,
        PredefinedUnderrideRuleId,
    },
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
    pub room: IndexSet<SimplePushRule<OwnedRoomId>>,

    /// These rules configure notification behavior for messages from a specific Matrix user ID.
    pub sender: IndexSet<SimplePushRule<OwnedUserId>>,

    /// These rules are identical to override rules, but have a lower priority than `content`,
    /// `room` and `sender` rules.
    pub underride: IndexSet<ConditionalPushRule>,
}

impl Ruleset {
    /// Creates an empty `Ruleset`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Creates a borrowing iterator over all push rules in this `Ruleset`.
    ///
    /// For an owning iterator, use `.into_iter()`.
    pub fn iter(&self) -> RulesetIter<'_> {
        self.into_iter()
    }

    /// Inserts a user-defined rule in the rule set.
    ///
    /// If a rule with the same kind and `rule_id` exists, it will be replaced.
    ///
    /// If `after` or `before` is set, the rule will be moved relative to the rule with the given
    /// ID. If both are set, the rule will become the next-most important rule with respect to
    /// `before`. If neither are set, and the rule is newly inserted, it will become the rule with
    /// the highest priority of its kind.
    ///
    /// Returns an error if the parameters are invalid.
    pub fn insert(
        &mut self,
        rule: NewPushRule,
        after: Option<&str>,
        before: Option<&str>,
    ) -> Result<(), InsertPushRuleError> {
        let rule_id = rule.rule_id();
        if rule_id.starts_with('.') {
            return Err(InsertPushRuleError::ServerDefaultRuleId);
        }
        if rule_id.contains('/') {
            return Err(InsertPushRuleError::InvalidRuleId);
        }
        if rule_id.contains('\\') {
            return Err(InsertPushRuleError::InvalidRuleId);
        }
        if after.map_or(false, |s| s.starts_with('.')) {
            return Err(InsertPushRuleError::RelativeToServerDefaultRule);
        }
        if before.map_or(false, |s| s.starts_with('.')) {
            return Err(InsertPushRuleError::RelativeToServerDefaultRule);
        }

        match rule {
            NewPushRule::Override(r) => {
                let mut rule = ConditionalPushRule::from(r);

                if let Some(prev_rule) = self.override_.get(rule.rule_id.as_str()) {
                    rule.enabled = prev_rule.enabled;
                }

                // `m.rule.master` should always be the rule with the highest priority, so we insert
                // this one at most at the second place.
                let default_position = 1;

                insert_and_move_rule(&mut self.override_, rule, default_position, after, before)
            }
            NewPushRule::Underride(r) => {
                let mut rule = ConditionalPushRule::from(r);

                if let Some(prev_rule) = self.underride.get(rule.rule_id.as_str()) {
                    rule.enabled = prev_rule.enabled;
                }

                insert_and_move_rule(&mut self.underride, rule, 0, after, before)
            }
            NewPushRule::Content(r) => {
                let mut rule = PatternedPushRule::from(r);

                if let Some(prev_rule) = self.content.get(rule.rule_id.as_str()) {
                    rule.enabled = prev_rule.enabled;
                }

                insert_and_move_rule(&mut self.content, rule, 0, after, before)
            }
            NewPushRule::Room(r) => {
                let mut rule = SimplePushRule::from(r);

                if let Some(prev_rule) = self.room.get(rule.rule_id.as_str()) {
                    rule.enabled = prev_rule.enabled;
                }

                insert_and_move_rule(&mut self.room, rule, 0, after, before)
            }
            NewPushRule::Sender(r) => {
                let mut rule = SimplePushRule::from(r);

                if let Some(prev_rule) = self.sender.get(rule.rule_id.as_str()) {
                    rule.enabled = prev_rule.enabled;
                }

                insert_and_move_rule(&mut self.sender, rule, 0, after, before)
            }
        }
    }

    /// Get the rule from the given kind and with the given `rule_id` in the rule set.
    pub fn get(&self, kind: RuleKind, rule_id: impl AsRef<str>) -> Option<AnyPushRuleRef<'_>> {
        let rule_id = rule_id.as_ref();

        match kind {
            RuleKind::Override => self.override_.get(rule_id).map(AnyPushRuleRef::Override),
            RuleKind::Underride => self.underride.get(rule_id).map(AnyPushRuleRef::Underride),
            RuleKind::Sender => self.sender.get(rule_id).map(AnyPushRuleRef::Sender),
            RuleKind::Room => self.room.get(rule_id).map(AnyPushRuleRef::Room),
            RuleKind::Content => self.content.get(rule_id).map(AnyPushRuleRef::Content),
            RuleKind::_Custom(_) => None,
        }
    }

    /// Set whether the rule from the given kind and with the given `rule_id` in the rule set is
    /// enabled.
    ///
    /// Returns an error if the rule can't be found.
    pub fn set_enabled(
        &mut self,
        kind: RuleKind,
        rule_id: impl AsRef<str>,
        enabled: bool,
    ) -> Result<(), RuleNotFoundError> {
        let rule_id = rule_id.as_ref();

        match kind {
            RuleKind::Override => {
                let mut rule = self.override_.get(rule_id).ok_or(RuleNotFoundError)?.clone();
                rule.enabled = enabled;
                self.override_.replace(rule);
            }
            RuleKind::Underride => {
                let mut rule = self.underride.get(rule_id).ok_or(RuleNotFoundError)?.clone();
                rule.enabled = enabled;
                self.underride.replace(rule);
            }
            RuleKind::Sender => {
                let mut rule = self.sender.get(rule_id).ok_or(RuleNotFoundError)?.clone();
                rule.enabled = enabled;
                self.sender.replace(rule);
            }
            RuleKind::Room => {
                let mut rule = self.room.get(rule_id).ok_or(RuleNotFoundError)?.clone();
                rule.enabled = enabled;
                self.room.replace(rule);
            }
            RuleKind::Content => {
                let mut rule = self.content.get(rule_id).ok_or(RuleNotFoundError)?.clone();
                rule.enabled = enabled;
                self.content.replace(rule);
            }
            RuleKind::_Custom(_) => return Err(RuleNotFoundError),
        }

        Ok(())
    }

    /// Set the actions of the rule from the given kind and with the given `rule_id` in the rule
    /// set.
    ///
    /// Returns an error if the rule can't be found.
    pub fn set_actions(
        &mut self,
        kind: RuleKind,
        rule_id: impl AsRef<str>,
        actions: Vec<Action>,
    ) -> Result<(), RuleNotFoundError> {
        let rule_id = rule_id.as_ref();

        match kind {
            RuleKind::Override => {
                let mut rule = self.override_.get(rule_id).ok_or(RuleNotFoundError)?.clone();
                rule.actions = actions;
                self.override_.replace(rule);
            }
            RuleKind::Underride => {
                let mut rule = self.underride.get(rule_id).ok_or(RuleNotFoundError)?.clone();
                rule.actions = actions;
                self.underride.replace(rule);
            }
            RuleKind::Sender => {
                let mut rule = self.sender.get(rule_id).ok_or(RuleNotFoundError)?.clone();
                rule.actions = actions;
                self.sender.replace(rule);
            }
            RuleKind::Room => {
                let mut rule = self.room.get(rule_id).ok_or(RuleNotFoundError)?.clone();
                rule.actions = actions;
                self.room.replace(rule);
            }
            RuleKind::Content => {
                let mut rule = self.content.get(rule_id).ok_or(RuleNotFoundError)?.clone();
                rule.actions = actions;
                self.content.replace(rule);
            }
            RuleKind::_Custom(_) => return Err(RuleNotFoundError),
        }

        Ok(())
    }

    /// Get the first push rule that applies to this event, if any.
    ///
    /// # Arguments
    ///
    /// * `event` - The raw JSON of a room message event.
    /// * `context` - The context of the message and room at the time of the event.
    #[instrument(skip_all, fields(context.room_id = %context.room_id))]
    pub fn get_match<T>(
        &self,
        event: &Raw<T>,
        context: &PushConditionRoomCtx,
    ) -> Option<AnyPushRuleRef<'_>> {
        let event = FlattenedJson::from_raw(event);

        if event.get("sender").map_or(false, |sender| sender == context.user_id) {
            // no need to look at the rules if the event was by the user themselves
            None
        } else {
            self.iter().find(|rule| rule.applies(&event, context))
        }
    }

    /// Get the push actions that apply to this event.
    ///
    /// Returns an empty slice if no push rule applies.
    ///
    /// # Arguments
    ///
    /// * `event` - The raw JSON of a room message event.
    /// * `context` - The context of the message and room at the time of the event.
    #[instrument(skip_all, fields(context.room_id = %context.room_id))]
    pub fn get_actions<T>(&self, event: &Raw<T>, context: &PushConditionRoomCtx) -> &[Action] {
        self.get_match(event, context).map(|rule| rule.actions()).unwrap_or(&[])
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
pub struct SimplePushRule<T> {
    /// Actions to determine if and how a notification is delivered for events matching this rule.
    pub actions: Vec<Action>,

    /// Whether this is a default rule, or has been set explicitly.
    pub default: bool,

    /// Whether the push rule is enabled or not.
    pub enabled: bool,

    /// The ID of this rule.
    ///
    /// This is generally the Matrix ID of the entity that it applies to.
    pub rule_id: T,
}

/// Initial set of fields of `SimplePushRule`.
///
/// This struct will not be updated even if additional fields are added to `SimplePushRule` in a new
/// (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct SimplePushRuleInit<T> {
    /// Actions to determine if and how a notification is delivered for events matching this rule.
    pub actions: Vec<Action>,

    /// Whether this is a default rule, or has been set explicitly.
    pub default: bool,

    /// Whether the push rule is enabled or not.
    pub enabled: bool,

    /// The ID of this rule.
    ///
    /// This is generally the Matrix ID of the entity that it applies to.
    pub rule_id: T,
}

impl<T> From<SimplePushRuleInit<T>> for SimplePushRule<T> {
    fn from(init: SimplePushRuleInit<T>) -> Self {
        let SimplePushRuleInit { actions, default, enabled, rule_id } = init;
        Self { actions, default, enabled, rule_id }
    }
}

// The following trait are needed to be able to make
// an IndexSet of the type

impl<T> Hash for SimplePushRule<T>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rule_id.hash(state);
    }
}

impl<T> PartialEq for SimplePushRule<T>
where
    T: PartialEq<T>,
{
    fn eq(&self, other: &Self) -> bool {
        self.rule_id == other.rule_id
    }
}

impl<T> Eq for SimplePushRule<T> where T: Eq {}

impl<T> Equivalent<SimplePushRule<T>> for str
where
    T: AsRef<str>,
{
    fn equivalent(&self, key: &SimplePushRule<T>) -> bool {
        self == key.rule_id.as_ref()
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

impl ConditionalPushRule {
    /// Check if the push rule applies to the event.
    ///
    /// # Arguments
    ///
    /// * `event` - The flattened JSON representation of a room message event.
    /// * `context` - The context of the room at the time of the event.
    pub fn applies(&self, event: &FlattenedJson, context: &PushConditionRoomCtx) -> bool {
        if !self.enabled {
            return false;
        }

        #[cfg(feature = "unstable-msc3932")]
        {
            // These 3 rules always apply.
            if self.rule_id != PredefinedOverrideRuleId::Master.as_ref()
                && self.rule_id != PredefinedOverrideRuleId::RoomNotif.as_ref()
                && self.rule_id != PredefinedOverrideRuleId::ContainsDisplayName.as_ref()
            {
                // Push rules which don't specify a `room_version_supports` condition are assumed
                // to not support extensible events and are therefore expected to be treated as
                // disabled when a room version does support extensible events.
                let room_supports_ext_ev =
                    context.supported_features.contains(&RoomVersionFeature::ExtensibleEvents);
                let rule_has_room_version_supports = self.conditions.iter().any(|condition| {
                    matches!(condition, PushCondition::RoomVersionSupports { .. })
                });

                if room_supports_ext_ev && !rule_has_room_version_supports {
                    return false;
                }
            }
        }

        self.conditions.iter().all(|cond| cond.applies(event, context))
    }
}

/// Initial set of fields of `ConditionalPushRule`.
///
/// This struct will not be updated even if additional fields are added to `ConditionalPushRule` in
/// a new (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
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

impl PatternedPushRule {
    /// Check if the push rule applies to the event.
    ///
    /// # Arguments
    ///
    /// * `event` - The flattened JSON representation of a room message event.
    /// * `context` - The context of the room at the time of the event.
    pub fn applies_to(
        &self,
        key: &str,
        event: &FlattenedJson,
        context: &PushConditionRoomCtx,
    ) -> bool {
        if event.get("sender").map_or(false, |sender| sender == context.user_id) {
            return false;
        }

        self.enabled && condition::check_event_match(event, key, &self.pattern, context)
    }
}

/// Initial set of fields of `PatterenedPushRule`.
///
/// This struct will not be updated even if additional fields are added to `PatterenedPushRule` in a
/// new (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
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

/// Information for a pusher using the Push Gateway API.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct HttpPusherData {
    /// The URL to use to send notifications to.
    ///
    /// Required if the pusher's kind is http.
    pub url: String,

    /// The format to use when sending notifications to the Push Gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<PushFormat>,

    /// iOS (+ macOS?) specific default payload that will be sent to apple push notification
    /// service.
    ///
    /// For more information, see [Sygnal docs][sygnal].
    ///
    /// [sygnal]: https://github.com/matrix-org/sygnal/blob/main/docs/applications.md#ios-applications-beware
    // Not specified, issue: https://github.com/matrix-org/matrix-spec/issues/921
    #[cfg(feature = "unstable-unspecified")]
    #[serde(default, skip_serializing_if = "JsonValue::is_null")]
    pub default_payload: JsonValue,
}

impl HttpPusherData {
    /// Creates a new `HttpPusherData` with the given URL.
    pub fn new(url: String) -> Self {
        Self {
            url,
            format: None,
            #[cfg(feature = "unstable-unspecified")]
            default_payload: JsonValue::default(),
        }
    }
}

/// A special format that the homeserver should use when sending notifications to a Push Gateway.
/// Currently, only `event_id_only` is supported, see the [Push Gateway API][spec].
///
/// [spec]: https://spec.matrix.org/v1.4/push-gateway-api/#homeserver-behaviour
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PushFormat {
    /// Require the homeserver to only send a reduced set of fields in the push.
    EventIdOnly,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The kinds of push rules that are available.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RuleKind {
    /// User-configured rules that override all other kinds.
    Override,

    /// Lowest priority user-defined rules.
    Underride,

    /// Sender-specific rules.
    Sender,

    /// Room-specific rules.
    Room,

    /// Content-specific rules.
    Content,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// A push rule to update or create.
#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum NewPushRule {
    /// Rules that override all other kinds.
    Override(NewConditionalPushRule),

    /// Content-specific rules.
    Content(NewPatternedPushRule),

    /// Room-specific rules.
    Room(NewSimplePushRule<OwnedRoomId>),

    /// Sender-specific rules.
    Sender(NewSimplePushRule<OwnedUserId>),

    /// Lowest priority rules.
    Underride(NewConditionalPushRule),
}

impl NewPushRule {
    /// The kind of this `NewPushRule`.
    pub fn kind(&self) -> RuleKind {
        match self {
            NewPushRule::Override(_) => RuleKind::Override,
            NewPushRule::Content(_) => RuleKind::Content,
            NewPushRule::Room(_) => RuleKind::Room,
            NewPushRule::Sender(_) => RuleKind::Sender,
            NewPushRule::Underride(_) => RuleKind::Underride,
        }
    }

    /// The ID of this `NewPushRule`.
    pub fn rule_id(&self) -> &str {
        match self {
            NewPushRule::Override(r) => &r.rule_id,
            NewPushRule::Content(r) => &r.rule_id,
            NewPushRule::Room(r) => r.rule_id.as_ref(),
            NewPushRule::Sender(r) => r.rule_id.as_ref(),
            NewPushRule::Underride(r) => &r.rule_id,
        }
    }
}

/// A simple push rule to update or create.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct NewSimplePushRule<T> {
    /// The ID of this rule.
    ///
    /// This is generally the Matrix ID of the entity that it applies to.
    pub rule_id: T,

    /// Actions to determine if and how a notification is delivered for events matching this
    /// rule.
    pub actions: Vec<Action>,
}

impl<T> NewSimplePushRule<T> {
    /// Creates a `NewSimplePushRule` with the given ID and actions.
    pub fn new(rule_id: T, actions: Vec<Action>) -> Self {
        Self { rule_id, actions }
    }
}

impl<T> From<NewSimplePushRule<T>> for SimplePushRule<T> {
    fn from(new_rule: NewSimplePushRule<T>) -> Self {
        let NewSimplePushRule { rule_id, actions } = new_rule;
        Self { actions, default: false, enabled: true, rule_id }
    }
}

/// A patterned push rule to update or create.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct NewPatternedPushRule {
    /// The ID of this rule.
    pub rule_id: String,

    /// The glob-style pattern to match against.
    pub pattern: String,

    /// Actions to determine if and how a notification is delivered for events matching this
    /// rule.
    pub actions: Vec<Action>,
}

impl NewPatternedPushRule {
    /// Creates a `NewPatternedPushRule` with the given ID, pattern and actions.
    pub fn new(rule_id: String, pattern: String, actions: Vec<Action>) -> Self {
        Self { rule_id, pattern, actions }
    }
}

impl From<NewPatternedPushRule> for PatternedPushRule {
    fn from(new_rule: NewPatternedPushRule) -> Self {
        let NewPatternedPushRule { rule_id, pattern, actions } = new_rule;
        Self { actions, default: false, enabled: true, rule_id, pattern }
    }
}

/// A conditional push rule to update or create.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct NewConditionalPushRule {
    /// The ID of this rule.
    pub rule_id: String,

    /// The conditions that must hold true for an event in order for a rule to be applied to an
    /// event.
    ///
    /// A rule with no conditions always matches.
    pub conditions: Vec<PushCondition>,

    /// Actions to determine if and how a notification is delivered for events matching this
    /// rule.
    pub actions: Vec<Action>,
}

impl NewConditionalPushRule {
    /// Creates a `NewConditionalPushRule` with the given ID, conditions and actions.
    pub fn new(rule_id: String, conditions: Vec<PushCondition>, actions: Vec<Action>) -> Self {
        Self { rule_id, conditions, actions }
    }
}

impl From<NewConditionalPushRule> for ConditionalPushRule {
    fn from(new_rule: NewConditionalPushRule) -> Self {
        let NewConditionalPushRule { rule_id, conditions, actions } = new_rule;
        Self { actions, default: false, enabled: true, rule_id, conditions }
    }
}

/// The error type returned when trying to insert a user-defined push rule into a `Ruleset`.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum InsertPushRuleError {
    /// The rule ID starts with a dot (`.`), which is reserved for server-default rules.
    #[error("rule IDs starting with a dot are reserved for server-default rules")]
    ServerDefaultRuleId,

    /// The rule ID contains an invalid character.
    #[error("invalid rule ID")]
    InvalidRuleId,

    /// The rule is being placed relative to a server-default rule, which is forbidden.
    #[error("can't place rule relative to server-default rule")]
    RelativeToServerDefaultRule,

    /// The `before` or `after` rule could not be found.
    #[error("The before or after rule could not be found")]
    UnknownRuleId,

    /// `before` has a higher priority than `after`.
    #[error("before has a higher priority than after")]
    BeforeHigherThanAfter,
}

/// The error type returned when trying modify a push rule that could not be found in a `Ruleset`.
#[derive(Debug, Error)]
#[non_exhaustive]
#[error("The rule could not be found")]
pub struct RuleNotFoundError;

/// Insert the rule in the given indexset and move it to the given position.
pub fn insert_and_move_rule<T>(
    set: &mut IndexSet<T>,
    rule: T,
    default_position: usize,
    after: Option<&str>,
    before: Option<&str>,
) -> Result<(), InsertPushRuleError>
where
    T: Hash + Eq,
    str: Equivalent<T>,
{
    let (from, replaced) = set.replace_full(rule);

    let mut to = default_position;

    if let Some(rule_id) = after {
        let idx = set.get_index_of(rule_id).ok_or(InsertPushRuleError::UnknownRuleId)?;
        to = idx + 1;
    }
    if let Some(rule_id) = before {
        let idx = set.get_index_of(rule_id).ok_or(InsertPushRuleError::UnknownRuleId)?;

        if idx < to {
            return Err(InsertPushRuleError::BeforeHigherThanAfter);
        }

        to = idx;
    }

    // Only move the item if it's new or if it was positioned.
    if replaced.is_none() || after.is_some() || before.is_some() {
        set.move_index(from, to);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use assert_matches::assert_matches;
    use js_int::{int, uint};
    use serde_json::{
        from_value as from_json_value, json, to_value as to_json_value,
        value::RawValue as RawJsonValue, Value as JsonValue,
    };

    use super::{
        action::{Action, Tweak},
        condition::{PushCondition, PushConditionRoomCtx, RoomMemberCountIs},
        AnyPushRule, ConditionalPushRule, PatternedPushRule, Ruleset, SimplePushRule,
    };
    use crate::{power_levels::NotificationPowerLevels, room_id, serde::Raw, user_id};

    fn example_ruleset() -> Ruleset {
        let mut set = Ruleset::new();

        set.override_.insert(ConditionalPushRule {
            conditions: vec![PushCondition::EventMatch {
                key: "type".into(),
                pattern: "m.call.invite".into(),
            }],
            actions: vec![Action::Notify, Action::SetTweak(Tweak::Highlight(true))],
            rule_id: ".m.rule.call".into(),
            enabled: true,
            default: true,
        });

        set
    }

    #[test]
    fn iter() {
        let mut set = example_ruleset();

        let added = set.override_.insert(ConditionalPushRule {
            conditions: vec![PushCondition::EventMatch {
                key: "room_id".into(),
                pattern: "!roomid:matrix.org".into(),
            }],
            actions: vec![Action::DontNotify],
            rule_id: "!roomid:matrix.org".into(),
            enabled: true,
            default: false,
        });
        assert!(added);

        let added = set.override_.insert(ConditionalPushRule {
            conditions: vec![],
            actions: vec![],
            rule_id: ".m.rule.suppress_notices".into(),
            enabled: false,
            default: true,
        });
        assert!(added);

        let mut iter = set.into_iter();

        let rule_opt = iter.next();
        assert!(rule_opt.is_some());
        let rule_id = assert_matches!(
            rule_opt.unwrap(),
            AnyPushRule::Override(ConditionalPushRule { rule_id, .. }) => rule_id
        );
        assert_eq!(rule_id, ".m.rule.call");

        let rule_opt = iter.next();
        assert!(rule_opt.is_some());
        let rule_id = assert_matches!(
            rule_opt.unwrap(),
            AnyPushRule::Override(ConditionalPushRule { rule_id, .. }) => rule_id
        );
        assert_eq!(rule_id, "!roomid:matrix.org");

        let rule_opt = iter.next();
        assert!(rule_opt.is_some());
        let rule_id = assert_matches!(
            rule_opt.unwrap(),
            AnyPushRule::Override(ConditionalPushRule { rule_id, .. }) => rule_id
        );
        assert_eq!(rule_id, ".m.rule.suppress_notices");

        assert_matches!(iter.next(), None);
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
            rule_id: room_id!("!roomid:server.name").to_owned(),
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

        set.override_.insert(ConditionalPushRule {
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
        });
        set.content.insert(PatternedPushRule {
            actions: vec![
                Action::Notify,
                Action::SetTweak(Tweak::Sound("default".into())),
                Action::SetTweak(Tweak::Highlight(true)),
            ],
            rule_id: ".m.rule.contains_user_name".into(),
            pattern: "user_id".into(),
            enabled: true,
            default: true,
        });

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
        let rule = from_json_value::<PatternedPushRule>(json!({
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
        assert!(rule.default);
        assert!(rule.enabled);
        assert_eq!(rule.pattern, "user_id");
        assert_eq!(rule.rule_id, ".m.rule.contains_user_name");

        let mut iter = rule.actions.iter();
        assert_matches!(iter.next(), Some(Action::Notify));
        let sound = assert_matches!(
            iter.next(),
            Some(Action::SetTweak(Tweak::Sound(sound))) => sound
        );
        assert_eq!(sound, "default");
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
        let rule_id = assert_matches!(
            rule_opt.unwrap(),
            AnyPushRule::Override(ConditionalPushRule { rule_id, .. }) => rule_id
        );
        assert_eq!(rule_id, "!roomid:server.name");

        let rule_opt = iter.next();
        assert!(rule_opt.is_some());
        let rule_id = assert_matches!(
            rule_opt.unwrap(),
            AnyPushRule::Override(ConditionalPushRule { rule_id, .. }) => rule_id
        );
        assert_eq!(rule_id, ".m.rule.call");

        let rule_opt = iter.next();
        assert!(rule_opt.is_some());
        let rule_id = assert_matches!(
            rule_opt.unwrap(),
            AnyPushRule::Content(PatternedPushRule { rule_id, .. }) => rule_id
        );
        assert_eq!(rule_id, ".m.rule.contains_user_name");

        let rule_opt = iter.next();
        assert!(rule_opt.is_some());
        let rule_id = assert_matches!(
            rule_opt.unwrap(),
            AnyPushRule::Content(PatternedPushRule { rule_id, .. }) => rule_id
        );
        assert_eq!(rule_id, "ruma");

        let rule_opt = iter.next();
        assert!(rule_opt.is_some());
        let rule_id = assert_matches!(
            rule_opt.unwrap(),
            AnyPushRule::Room(SimplePushRule { rule_id, .. }) => rule_id
        );
        assert_eq!(rule_id, "!roomid:server.name");

        let rule_opt = iter.next();
        assert!(rule_opt.is_some());
        let rule_id = assert_matches!(
            rule_opt.unwrap(),
            AnyPushRule::Underride(ConditionalPushRule { rule_id, .. }) => rule_id
        );
        assert_eq!(rule_id, ".m.rule.room_one_to_one");

        assert_matches!(iter.next(), None);
    }

    #[test]
    fn default_ruleset_applies() {
        let set = Ruleset::server_default(user_id!("@jolly_jumper:server.name"));

        let context_one_to_one = &PushConditionRoomCtx {
            room_id: room_id!("!dm:server.name").to_owned(),
            member_count: uint!(2),
            user_id: user_id!("@jj:server.name").to_owned(),
            user_display_name: "Jolly Jumper".into(),
            users_power_levels: BTreeMap::new(),
            default_power_level: int!(50),
            notification_power_levels: NotificationPowerLevels { room: int!(50) },
            #[cfg(feature = "unstable-msc3931")]
            supported_features: Default::default(),
        };

        let context_public_room = &PushConditionRoomCtx {
            room_id: room_id!("!far_west:server.name").to_owned(),
            member_count: uint!(100),
            user_id: user_id!("@jj:server.name").to_owned(),
            user_display_name: "Jolly Jumper".into(),
            users_power_levels: BTreeMap::new(),
            default_power_level: int!(50),
            notification_power_levels: NotificationPowerLevels { room: int!(50) },
            #[cfg(feature = "unstable-msc3931")]
            supported_features: Default::default(),
        };

        let message = serde_json::from_str::<Raw<JsonValue>>(
            r#"{
                "type": "m.room.message"
            }"#,
        )
        .unwrap();

        assert_matches!(
            set.get_actions(&message, context_one_to_one),
            [
                Action::Notify,
                Action::SetTweak(Tweak::Sound(_)),
                Action::SetTweak(Tweak::Highlight(false))
            ]
        );
        assert_matches!(
            set.get_actions(&message, context_public_room),
            [Action::Notify, Action::SetTweak(Tweak::Highlight(false))]
        );

        let user_name = serde_json::from_str::<Raw<JsonValue>>(
            r#"{
                "type": "m.room.message",
                "content": {
                    "body": "Hi jolly_jumper!"
                }
            }"#,
        )
        .unwrap();

        assert_matches!(
            set.get_actions(&user_name, context_one_to_one),
            [
                Action::Notify,
                Action::SetTweak(Tweak::Sound(_)),
                Action::SetTweak(Tweak::Highlight(true)),
            ]
        );
        assert_matches!(
            set.get_actions(&user_name, context_public_room),
            [
                Action::Notify,
                Action::SetTweak(Tweak::Sound(_)),
                Action::SetTweak(Tweak::Highlight(true)),
            ]
        );

        let notice = serde_json::from_str::<Raw<JsonValue>>(
            r#"{
                "type": "m.room.message",
                "content": {
                    "msgtype": "m.notice"
                }
            }"#,
        )
        .unwrap();
        assert_matches!(set.get_actions(&notice, context_one_to_one), [Action::DontNotify]);

        let at_room = serde_json::from_str::<Raw<JsonValue>>(
            r#"{
                "type": "m.room.message",
                "sender": "@rantanplan:server.name",
                "content": {
                    "body": "@room Attention please!",
                    "msgtype": "m.text"
                }
            }"#,
        )
        .unwrap();

        assert_matches!(
            set.get_actions(&at_room, context_public_room),
            [Action::Notify, Action::SetTweak(Tweak::Highlight(true)),]
        );

        let empty = serde_json::from_str::<Raw<JsonValue>>(r#"{}"#).unwrap();
        assert_matches!(set.get_actions(&empty, context_one_to_one), []);
    }

    #[test]
    fn custom_ruleset_applies() {
        let context_one_to_one = &PushConditionRoomCtx {
            room_id: room_id!("!dm:server.name").to_owned(),
            member_count: uint!(2),
            user_id: user_id!("@jj:server.name").to_owned(),
            user_display_name: "Jolly Jumper".into(),
            users_power_levels: BTreeMap::new(),
            default_power_level: int!(50),
            notification_power_levels: NotificationPowerLevels { room: int!(50) },
            #[cfg(feature = "unstable-msc3931")]
            supported_features: Default::default(),
        };

        let message = serde_json::from_str::<Raw<JsonValue>>(
            r#"{
                "sender": "@rantanplan:server.name",
                "type": "m.room.message",
                "content": {
                    "msgtype": "m.text",
                    "body": "Great joke!"
                }
            }"#,
        )
        .unwrap();

        let mut set = Ruleset::new();
        let disabled = ConditionalPushRule {
            actions: vec![Action::Notify],
            default: false,
            enabled: false,
            rule_id: "disabled".into(),
            conditions: vec![PushCondition::RoomMemberCount {
                is: RoomMemberCountIs::from(uint!(2)),
            }],
        };
        set.underride.insert(disabled);

        let test_set = set.clone();
        assert_matches!(test_set.get_actions(&message, context_one_to_one), []);

        let no_conditions = ConditionalPushRule {
            actions: vec![Action::SetTweak(Tweak::Highlight(true))],
            default: false,
            enabled: true,
            rule_id: "no.conditions".into(),
            conditions: vec![],
        };
        set.underride.insert(no_conditions);

        let test_set = set.clone();
        assert_matches!(
            test_set.get_actions(&message, context_one_to_one),
            [Action::SetTweak(Tweak::Highlight(true))]
        );

        let sender = SimplePushRule {
            actions: vec![Action::Notify],
            default: false,
            enabled: true,
            rule_id: user_id!("@rantanplan:server.name").to_owned(),
        };
        set.sender.insert(sender);

        let test_set = set.clone();
        assert_matches!(test_set.get_actions(&message, context_one_to_one), [Action::Notify]);

        let room = SimplePushRule {
            actions: vec![Action::DontNotify],
            default: false,
            enabled: true,
            rule_id: room_id!("!dm:server.name").to_owned(),
        };
        set.room.insert(room);

        let test_set = set.clone();
        assert_matches!(test_set.get_actions(&message, context_one_to_one), [Action::DontNotify]);

        let content = PatternedPushRule {
            actions: vec![Action::SetTweak(Tweak::Sound("content".into()))],
            default: false,
            enabled: true,
            rule_id: "content".into(),
            pattern: "joke".into(),
        };
        set.content.insert(content);

        let test_set = set.clone();
        assert_matches!(
            test_set.get_actions(&message, context_one_to_one),
            [Action::SetTweak(Tweak::Sound(sound))]
            if sound == "content"
        );

        let three_conditions = ConditionalPushRule {
            actions: vec![Action::SetTweak(Tweak::Sound("three".into()))],
            default: false,
            enabled: true,
            rule_id: "three.conditions".into(),
            conditions: vec![
                PushCondition::RoomMemberCount { is: RoomMemberCountIs::from(uint!(2)) },
                PushCondition::ContainsDisplayName,
                PushCondition::EventMatch {
                    key: "room_id".into(),
                    pattern: "!dm:server.name".into(),
                },
            ],
        };
        set.override_.insert(three_conditions);

        let sound = assert_matches!(
            set.get_actions(&message, context_one_to_one),
            [Action::SetTweak(Tweak::Sound(sound))] => sound
        );
        assert_eq!(sound, "content");

        let new_message = serde_json::from_str::<Raw<JsonValue>>(
            r#"{
                "sender": "@rantanplan:server.name",
                "type": "m.room.message",
                "content": {
                    "msgtype": "m.text",
                    "body": "Tell me another one, Jolly Jumper!"
                }
            }"#,
        )
        .unwrap();

        let sound = assert_matches!(
            set.get_actions(&new_message, context_one_to_one),
            [Action::SetTweak(Tweak::Sound(sound))] => sound
        );
        assert_eq!(sound, "three");
    }
}
