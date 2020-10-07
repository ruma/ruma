//! Types for the *m.policy.rule.room* event.

use ruma_events_macros::StateEventContent;

use serde::{Deserialize, Serialize};

use crate::{policy::rule::PolicyRuleEventContent, StateEvent};

/// This event type is used to apply rules to room entities.
pub type RoomEvent = StateEvent<RoomEventContent>;

/// The payload for `RoomEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[ruma_event(type = "m.policy.rule.room")]
pub struct RoomEventContent(PolicyRuleEventContent);
