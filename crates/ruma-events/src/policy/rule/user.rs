//! Types for the [`m.policy.rule.user`] event.
//!
//! [`m.policy.rule.user`]: https://spec.matrix.org/latest/client-server-api/#mpolicyruleuser

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{PolicyRuleEventContent, PossiblyRedactedPolicyRuleEventContent};
use crate::{EventContent, PossiblyRedactedStateEventContent, StateEventType};

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

impl EventContent for PossiblyRedactedPolicyRuleUserEventContent {
    type EventType = StateEventType;

    fn event_type(&self) -> Self::EventType {
        StateEventType::PolicyRuleUser
    }
}

impl PossiblyRedactedStateEventContent for PossiblyRedactedPolicyRuleUserEventContent {
    type StateKey = String;
}
