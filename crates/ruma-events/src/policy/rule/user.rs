//! Types for the [`m.policy.rule.user`] event.
//!
//! [`m.policy.rule.user`]: https://spec.matrix.org/v1.19/client-server-api/#mpolicyruleuser

use ruma_common::room_version_rules::RedactionRules;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::PolicyRuleEventContent;
use crate::{RedactContent, RedactedStateEventContent};

/// The content of an `m.policy.rule.user` event.
///
/// This event type is used to apply rules to user entities.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[allow(clippy::exhaustive_structs)]
#[ruma_event(type = "m.policy.rule.user", kind = State, state_key_type = String, custom_redacted)]
pub struct PolicyRuleUserEventContent(pub PolicyRuleEventContent);

impl RedactContent for PolicyRuleUserEventContent {
    type Redacted = Self;

    fn redact(self, _rules: &RedactionRules) -> Self::Redacted {
        Self(PolicyRuleEventContent::empty())
    }
}

impl RedactedStateEventContent for PolicyRuleUserEventContent {
    type StateKey = String;

    fn event_type(&self) -> crate::StateEventType {
        crate::StateEventType::PolicyRuleUser
    }
}
