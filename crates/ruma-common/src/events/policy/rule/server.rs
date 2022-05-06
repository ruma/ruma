//! Types for the [`m.policy.rule.server`] event.
//!
//! [`m.policy.rule.server`]: https://spec.matrix.org/v1.2/client-server-api/#mpolicyruleserver

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::PolicyRuleEventContent;

/// The content of an `m.policy.rule.server` event.
///
/// This event type is used to apply rules to server entities.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[allow(clippy::exhaustive_structs)]
#[ruma_event(type = "m.policy.rule.server", kind = State, state_key_type = String)]
pub struct PolicyRuleServerEventContent(pub PolicyRuleEventContent);
