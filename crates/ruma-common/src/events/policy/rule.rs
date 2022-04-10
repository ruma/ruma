//! Modules and types for events in the `m.policy.rule` namespace.

use serde::{Deserialize, Serialize};

use crate::{serde::StringEnum, PrivOwnedStr};

pub mod room;
pub mod server;
pub mod user;

/// The payload for policy rule events.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PolicyRuleEventContent {
    /// The entity affected by this rule.
    ///
    /// Glob characters `*` and `?` can be used to match zero or more and one or more characters
    /// respectively.
    pub entity: String,

    /// The suggested action to take.
    pub recommendation: Recommendation,

    /// The human-readable description for the recommendation.
    pub reason: String,
}

impl PolicyRuleEventContent {
    /// Creates a new `PolicyRuleEventContent` with the given entity, recommendation and reason.
    pub fn new(entity: String, recommendation: Recommendation, reason: String) -> Self {
        Self { entity, recommendation, reason }
    }
}

/// The possible actions that can be taken.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[non_exhaustive]
pub enum Recommendation {
    /// Entities affected by the rule should be banned from participation where possible.
    #[ruma_enum(rename = "m.ban")]
    Ban,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl Recommendation {
    /// Creates a string slice from this `Recommendation`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}
