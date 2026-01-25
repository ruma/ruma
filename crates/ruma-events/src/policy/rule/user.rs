//! Types for the [`m.policy.rule.user`] event.
//!
//! [`m.policy.rule.user`]: https://spec.matrix.org/latest/client-server-api/#mpolicyruleuser

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{PolicyRuleEventContent, PossiblyRedactedPolicyRuleEventContent};
use crate::{PossiblyRedactedStateEventContent, StateEventType, StaticEventContent};

/// The content of an `m.policy.rule.user` event.
///
/// This event type is used to apply rules to user entities.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[allow(clippy::exhaustive_structs)]
#[ruma_event(type = "m.policy.rule.user", kind = State, state_key_type = String, custom_possibly_redacted)]
pub struct PolicyRuleUserEventContent(pub PolicyRuleEventContent);

/// The possibly redacted form of [`PolicyRuleUserEventContent`].
///
/// This type is used when it's not obvious whether the content is redacted or not.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::exhaustive_structs)]
pub struct PossiblyRedactedPolicyRuleUserEventContent(pub PossiblyRedactedPolicyRuleEventContent);

impl PossiblyRedactedStateEventContent for PossiblyRedactedPolicyRuleUserEventContent {
    type StateKey = String;

    fn event_type(&self) -> StateEventType {
        StateEventType::PolicyRuleUser
    }
}

impl StaticEventContent for PossiblyRedactedPolicyRuleUserEventContent {
    const TYPE: &'static str = PolicyRuleUserEventContent::TYPE;
    type IsPrefix = <PolicyRuleUserEventContent as StaticEventContent>::IsPrefix;
}

impl From<PolicyRuleUserEventContent> for PossiblyRedactedPolicyRuleUserEventContent {
    fn from(value: PolicyRuleUserEventContent) -> Self {
        let PolicyRuleUserEventContent(policy) = value;
        Self(policy.into())
    }
}

impl From<RedactedPolicyRuleUserEventContent> for PossiblyRedactedPolicyRuleUserEventContent {
    fn from(value: RedactedPolicyRuleUserEventContent) -> Self {
        let RedactedPolicyRuleUserEventContent {} = value;
        Self(PossiblyRedactedPolicyRuleEventContent::empty())
    }
}
