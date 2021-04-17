use indexmap::set::{IntoIter as IndexSetIntoIter, Iter as IndexSetIter};

use super::{ConditionalPushRule, PatternedPushRule, Ruleset, SimplePushRule};

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
    /// Get the `rule_id` of the push rule.
    pub fn rule_id(&self) -> &str {
        match self {
            Self::Override(rule) => &rule.rule_id,
            Self::Underride(rule) => &rule.rule_id,
            Self::Content(rule) => &rule.rule_id,
            Self::Room(rule) => &rule.rule_id,
            Self::Sender(rule) => &rule.rule_id,
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
