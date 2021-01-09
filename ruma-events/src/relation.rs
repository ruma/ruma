//! Types describing event relations after MSC 2674, 2675, 2676, 2677.

use std::{fmt::Debug, time::SystemTime};

use js_int::UInt;
use serde::{Deserialize, Serialize};

/// Summary of all reactions with the given key to an event.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct BundledReaction {
    /// The key (emoji) used for reaction.
    pub key: String,

    /// Time of the bundled reaction being compiled on the server.
    #[serde(
        with = "ruma_serde::time::opt_ms_since_unix_epoch",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub origin_server_ts: Option<SystemTime>,

    /// Number of reactions.
    pub count: UInt,
}

impl BundledReaction {
    /// Creates a new `BundledReaction`.
    pub fn new(key: String, origin_server_ts: Option<SystemTime>, count: UInt) -> Self {
        Self { key, origin_server_ts, count }
    }
}

/// Type of bundled annotation.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type")]
pub enum BundledAnnotation {
    /// An emoji reaction and its count.
    #[serde(rename = "m.reaction")]
    Reaction(BundledReaction),
}

/// The first chunk of annotations with a token for loading more.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct AnnotationChunk {
    /// The first batch of bundled annotations.
    pub chunk: Vec<BundledAnnotation>,

    /// Token to receive the next annotation batch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_batch: Option<String>,
}

impl AnnotationChunk {
    /// Creates a new `AnnotationChunk`.
    pub fn new(chunk: Vec<BundledAnnotation>, next_batch: Option<String>) -> Self {
        Self { chunk, next_batch }
    }
}

/// Precompiled list of relations to this event grouped by relation type.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Relations {
    /// Annotation relations.
    #[serde(rename = "m.annotation")]
    pub annotation: Option<AnnotationChunk>,
}

impl Relations {
    /// Creates a new `Relations` struct.
    pub fn new(annotation: Option<AnnotationChunk>) -> Self {
        Self { annotation }
    }
}
