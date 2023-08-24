//! Types for the [`m.policy.rule.server`] event.
//!
//! [`m.policy.rule.server`]: https://spec.matrix.org/latest/client-server-api/#mpolicyruleserver

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{PolicyRuleEventContent, PossiblyRedactedPolicyRuleEventContent};
use crate::{EventContent, PossiblyRedactedStateEventContent, StateEventType};

/// The content of an `m.policy.rule.server` event.
///
/// This event type is used to apply rules to server entities.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[allow(clippy::exhaustive_structs)]
#[ruma_event(type = "m.policy.rule.server", kind = State, state_key_type = String, custom_possibly_redacted)]
pub struct PolicyRuleServerEventContent(pub PolicyRuleEventContent);

/// The possibly redacted form of [`PolicyRuleServerEventContent`].
///
/// This type is used when it's not obvious whether the content is redacted or not.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::exhaustive_structs)]
pub struct PossiblyRedactedPolicyRuleServerEventContent(pub PossiblyRedactedPolicyRuleEventContent);

impl EventContent for PossiblyRedactedPolicyRuleServerEventContent {
    type EventType = StateEventType;

    fn event_type(&self) -> Self::EventType {
        StateEventType::PolicyRuleServer
    }
}

impl PossiblyRedactedStateEventContent for PossiblyRedactedPolicyRuleServerEventContent {
    type StateKey = String;
}
