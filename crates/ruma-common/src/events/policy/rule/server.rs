//! Types for the [`m.policy.rule.server`] event.
//!
//! [`m.policy.rule.server`]: https://spec.matrix.org/v1.4/client-server-api/#mpolicyruleserver

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;

use super::{PolicyRuleEventContent, PossiblyRedactedPolicyRuleEventContent};
use crate::events::{EventContent, StateEventContent, StateEventType};

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

    fn from_parts(event_type: &str, content: &RawJsonValue) -> serde_json::Result<Self> {
        if event_type != "m.policy.rule.server" {
            return Err(::serde::de::Error::custom(format!(
                "expected event type `m.policy.rule.server`, found `{event_type}`",
            )));
        }

        serde_json::from_str(content.get())
    }
}

impl StateEventContent for PossiblyRedactedPolicyRuleServerEventContent {
    type StateKey = String;
}
