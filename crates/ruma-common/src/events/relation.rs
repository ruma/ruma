//! Types describing event relations after MSC 2674, 2675, 2676, 2677.

use std::fmt::Debug;

use js_int::UInt;
use serde::{Deserialize, Serialize};

use crate::{EventId, MilliSecondsSinceUnixEpoch, UserId};

/// Summary of all reactions with the given key to an event.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[cfg(feature = "unstable-msc2677")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct BundledReaction {
    /// The key (emoji) used for reaction.
    pub key: String,

    /// Time of the bundled reaction being compiled on the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_server_ts: Option<MilliSecondsSinceUnixEpoch>,

    /// Number of reactions.
    pub count: UInt,
}

#[cfg(feature = "unstable-msc2677")]
impl BundledReaction {
    /// Creates a new `BundledReaction`.
    pub fn new(
        key: String,
        origin_server_ts: Option<MilliSecondsSinceUnixEpoch>,
        count: UInt,
    ) -> Self {
        Self { key, origin_server_ts, count }
    }
}

/// Type of bundled annotation.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg(feature = "unstable-msc2677")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type")]
pub enum BundledAnnotation {
    /// An emoji reaction and its count.
    #[serde(rename = "m.reaction")]
    Reaction(BundledReaction),
}

/// The first chunk of annotations with a token for loading more.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg(feature = "unstable-msc2677")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct AnnotationChunk {
    /// The first batch of bundled annotations.
    pub chunk: Vec<BundledAnnotation>,

    /// Token to receive the next annotation batch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_batch: Option<String>,
}

#[cfg(feature = "unstable-msc2677")]
impl AnnotationChunk {
    /// Creates a new `AnnotationChunk` with the given chunk and next batch token.
    pub fn new(chunk: Vec<BundledAnnotation>, next_batch: Option<String>) -> Self {
        Self { chunk, next_batch }
    }
}

/// A bundled replacement.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg(feature = "unstable-msc2676")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct BundledReplacement {
    /// The ID of the replacing event.
    pub event_id: Box<EventId>,

    /// The user ID of the sender of the latest replacement.
    pub sender: Box<UserId>,

    /// Timestamp in milliseconds on originating homeserver when the latest replacement was sent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_server_ts: Option<MilliSecondsSinceUnixEpoch>,
}

#[cfg(feature = "unstable-msc2676")]
impl BundledReplacement {
    /// Creates a new `BundledReplacement` with the given event ID and sender.
    pub fn new(event_id: Box<EventId>, sender: Box<UserId>) -> Self {
        Self { event_id, sender, origin_server_ts: None }
    }
}

/// Precompiled list of relations to this event grouped by relation type.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Relations {
    /// Annotation relations.
    #[cfg(feature = "unstable-msc2677")]
    #[serde(rename = "m.annotation")]
    pub annotation: Option<AnnotationChunk>,

    /// Replacement relation.
    #[cfg(feature = "unstable-msc2676")]
    #[serde(rename = "m.replace")]
    pub replace: Option<BundledReplacement>,
}

impl Relations {
    /// Creates a new empty `Relations`.
    pub fn new() -> Self {
        Self::default()
    }
}
