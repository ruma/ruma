//! Types describing event relations after MSC 2674, 2675, 2676, 2677.

use std::fmt::Debug;

use js_int::UInt;
use ruma_common::MilliSecondsSinceUnixEpoch;
use serde::{Deserialize, Serialize};

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

/// Precompiled list of relations to this event grouped by relation type.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Relations {
    /// Annotation relations.
    #[cfg(feature = "unstable-msc2677")]
    #[serde(rename = "m.annotation")]
    pub annotation: Option<AnnotationChunk>,
}

impl Relations {
    #[cfg(feature = "unstable-msc2677")]
    /// Creates a new `Relations` with the given annotation.
    ///
    /// Without the `unstable-msc-2677` feature, this method doesn't have any
    /// parameters.
    pub fn new(annotation: Option<AnnotationChunk>) -> Self {
        Self { annotation }
    }

    #[cfg(not(feature = "unstable-msc2677"))]
    /// Creates a new empty `Relations`.
    ///
    /// With the `unstable-msc-2677` feature, this method takes an annotation.
    pub fn new() -> Self {
        Self {}
    }
}
