//! Types for the [`m.policy.rule.user`] event.
//!
//! [`m.policy.rule.user`]: https://spec.matrix.org/v1.2/client-server-api/#mpolicyruleuser

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::PolicyRuleEventContent;

/// The content of an `m.policy.rule.user` event.
///
/// This event type is used to apply rules to user entities.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[allow(clippy::exhaustive_structs)]
#[ruma_event(type = "m.policy.rule.user", kind = State, state_key_type = String)]
pub struct PolicyRuleUserEventContent(pub PolicyRuleEventContent);
