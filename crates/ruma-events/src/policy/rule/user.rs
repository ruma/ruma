//! Types for the *m.policy.rule.user* event.

use ruma_events_macros::StateEventContent;
use serde::{Deserialize, Serialize};

use crate::{policy::rule::PolicyRuleEventContent, StateEvent};

/// This event type is used to apply rules to user entities.
pub type UserEvent = StateEvent<UserEventContent>;

/// The payload for `UserEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[ruma_event(type = "m.policy.rule.user")]
pub struct UserEventContent(pub PolicyRuleEventContent);
