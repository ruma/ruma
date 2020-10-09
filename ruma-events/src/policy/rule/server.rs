//! Types for the *m.policy.rule.server* event.

use ruma_events_macros::StateEventContent;
use serde::{Deserialize, Serialize};

use crate::{policy::rule::PolicyRuleEventContent, StateEvent};

/// This event type is used to apply rules to server entities.
pub type ServerEvent = StateEvent<ServerEventContent>;

/// The payload for `ServerEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[ruma_event(type = "m.policy.rule.server")]
pub struct ServerEventContent(pub PolicyRuleEventContent);
