//! Types for the [`m.policy.rule.user`] event.
//!
//! [`m.policy.rule.user`]: https://spec.matrix.org/v1.4/client-server-api/#mpolicyruleuser

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;

use super::{PolicyRuleEventContent, PossiblyRedactedPolicyRuleEventContent};
use crate::events::{EventContent, StateEventContent, StateEventType};

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

    fn from_parts(event_type: &str, content: &RawJsonValue) -> serde_json::Result<Self> {
        if event_type != "m.policy.rule.user" {
            return Err(::serde::de::Error::custom(format!(
                "expected event type `m.policy.rule.user`, found `{event_type}`",
            )));
        }

        serde_json::from_str(content.get())
    }
}

impl StateEventContent for PossiblyRedactedPolicyRuleUserEventContent {
    type StateKey = String;
}
