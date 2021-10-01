//! Types for the `m.policy.rule.server` event.

use ruma_events_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::policy::rule::PolicyRuleEventContent;

/// The content of an `m.policy.rule.server` event.
///
/// This event type is used to apply rules to server entities.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[allow(clippy::exhaustive_structs)]
#[ruma_event(type = "m.policy.rule.server", kind = State)]
pub struct ServerEventContent(pub PolicyRuleEventContent);
