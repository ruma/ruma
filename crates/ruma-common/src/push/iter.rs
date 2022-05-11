use indexmap::set::{IntoIter as IndexSetIntoIter, Iter as IndexSetIter};

use super::{
    condition, Action, ConditionalPushRule, FlattenedJson, PatternedPushRule, PushConditionRoomCtx,
    Ruleset, SimplePushRule,
};

/// The kinds of push rules that are available.
#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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
    /// Convert `AnyPushRule` to `AnyPushRuleRef`.
    pub fn as_ref(&self) -> AnyPushRuleRef<'_> {
        match self {
            Self::Override(o) => AnyPushRuleRef::Override(o),
            Self::Content(c) => AnyPushRuleRef::Content(c),
            Self::Room(r) => AnyPushRuleRef::Room(r),
            Self::Sender(s) => AnyPushRuleRef::Sender(s),
            Self::Underride(u) => AnyPushRuleRef::Underride(u),
        }
    }

    /// Get the `enabled` flag of the push rule.
    pub fn enabled(&self) -> bool {
        self.as_ref().enabled()
    }

    /// Get the `actions` of the push rule.
    pub fn actions(&self) -> &[Action] {
        self.as_ref().actions()
    }

    /// Get the `rule_id` of the push rule.
    pub fn rule_id(&self) -> &str {
        self.as_ref().rule_id()
    }

    /// Check if the push rule applies to the event.
    ///
    /// # Arguments
    ///
    /// * `event` - The flattened JSON representation of a room message event.
    /// * `context` - The context of the room at the time of the event.
    pub fn applies(&self, event: &FlattenedJson, context: &PushConditionRoomCtx) -> bool {
        self.as_ref().applies(event, context)
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

/// Iterator type for `Ruleset`
#[derive(Debug)]
pub struct RulesetIntoIter {
    content: IndexSetIntoIter<PatternedPushRule>,
    override_: IndexSetIntoIter<ConditionalPushRule>,
    room: IndexSetIntoIter<SimplePushRule>,
    sender: IndexSetIntoIter<SimplePushRule>,
    underride: IndexSetIntoIter<ConditionalPushRule>,
}

impl Iterator for RulesetIntoIter {
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
    type IntoIter = RulesetIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        RulesetIntoIter {
            content: self.content.into_iter(),
            override_: self.override_.into_iter(),
            room: self.room.into_iter(),
            sender: self.sender.into_iter(),
            underride: self.underride.into_iter(),
        }
    }
}

/// Reference to any kind of push rule.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum AnyPushRuleRef<'a> {
    /// Rules that override all other kinds.
    Override(&'a ConditionalPushRule),

    /// Content-specific rules.
    Content(&'a PatternedPushRule),

    /// Room-specific rules.
    Room(&'a SimplePushRule),

    /// Sender-specific rules.
    Sender(&'a SimplePushRule),

    /// Lowest priority rules.
    Underride(&'a ConditionalPushRule),
}

impl<'a> AnyPushRuleRef<'a> {
    /// Convert `AnyPushRuleRef` to `AnyPushRule` by cloning the inner value.
    pub fn to_owned(self) -> AnyPushRule {
        match self {
            Self::Override(o) => AnyPushRule::Override(o.clone()),
            Self::Content(c) => AnyPushRule::Content(c.clone()),
            Self::Room(r) => AnyPushRule::Room(r.clone()),
            Self::Sender(s) => AnyPushRule::Sender(s.clone()),
            Self::Underride(u) => AnyPushRule::Underride(u.clone()),
        }
    }

    /// Get the `enabled` flag of the push rule.
    pub fn enabled(self) -> bool {
        match self {
            Self::Override(rule) => rule.enabled,
            Self::Underride(rule) => rule.enabled,
            Self::Content(rule) => rule.enabled,
            Self::Room(rule) => rule.enabled,
            Self::Sender(rule) => rule.enabled,
        }
    }

    /// Get the `actions` of the push rule.
    pub fn actions(self) -> &'a [Action] {
        match self {
            Self::Override(rule) => &rule.actions,
            Self::Underride(rule) => &rule.actions,
            Self::Content(rule) => &rule.actions,
            Self::Room(rule) => &rule.actions,
            Self::Sender(rule) => &rule.actions,
        }
    }

    /// Get the `rule_id` of the push rule.
    pub fn rule_id(self) -> &'a str {
        match self {
            Self::Override(rule) => &rule.rule_id,
            Self::Underride(rule) => &rule.rule_id,
            Self::Content(rule) => &rule.rule_id,
            Self::Room(rule) => &rule.rule_id,
            Self::Sender(rule) => &rule.rule_id,
        }
    }

    /// Check if the push rule applies to the event.
    ///
    /// # Arguments
    ///
    /// * `event` - The flattened JSON representation of a room message event.
    /// * `context` - The context of the room at the time of the event.
    pub fn applies(self, event: &FlattenedJson, context: &PushConditionRoomCtx) -> bool {
        if event.get("sender").map_or(false, |sender| sender == context.user_id) {
            return false;
        }

        match self {
            Self::Override(rule) => rule.applies(event, context),
            Self::Underride(rule) => rule.applies(event, context),
            Self::Content(rule) => rule.applies_to("content.body", event, context),
            Self::Room(rule) => {
                rule.enabled
                    && condition::check_event_match(event, "room_id", &rule.rule_id, context)
            }
            Self::Sender(rule) => {
                rule.enabled
                    && condition::check_event_match(event, "sender", &rule.rule_id, context)
            }
        }
    }
}

/// Iterator type for `Ruleset`
#[derive(Debug)]
pub struct RulesetIter<'a> {
    content: IndexSetIter<'a, PatternedPushRule>,
    override_: IndexSetIter<'a, ConditionalPushRule>,
    room: IndexSetIter<'a, SimplePushRule>,
    sender: IndexSetIter<'a, SimplePushRule>,
    underride: IndexSetIter<'a, ConditionalPushRule>,
}

impl<'a> Iterator for RulesetIter<'a> {
    type Item = AnyPushRuleRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.override_
            .next()
            .map(AnyPushRuleRef::Override)
            .or_else(|| self.content.next().map(AnyPushRuleRef::Content))
            .or_else(|| self.room.next().map(AnyPushRuleRef::Room))
            .or_else(|| self.sender.next().map(AnyPushRuleRef::Sender))
            .or_else(|| self.underride.next().map(AnyPushRuleRef::Underride))
    }
}

impl<'a> IntoIterator for &'a Ruleset {
    type Item = AnyPushRuleRef<'a>;
    type IntoIter = RulesetIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        RulesetIter {
            content: self.content.iter(),
            override_: self.override_.iter(),
            room: self.room.iter(),
            sender: self.sender.iter(),
            underride: self.underride.iter(),
        }
    }
}
