//! Types describing [relationships between events].
//!
//! [relationships between events]: https://spec.matrix.org/v1.3/client-server-api/#forming-relationships-between-events

use std::fmt::Debug;

use js_int::UInt;
use serde::{Deserialize, Serialize};

use super::AnyMessageLikeEvent;
#[cfg(any(feature = "unstable-msc2676", feature = "unstable-msc2677"))]
use crate::MilliSecondsSinceUnixEpoch;
use crate::{
    serde::{Raw, StringEnum},
    PrivOwnedStr,
};
#[cfg(feature = "unstable-msc2676")]
use crate::{OwnedEventId, OwnedUserId};

/// Summary of all annotations to an event with the given key and type.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[cfg(feature = "unstable-msc2677")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct BundledAnnotation {
    /// The type of the annotation.
    #[serde(rename = "type")]
    pub annotation_type: AnnotationType,

    /// The key used for the annotation.
    pub key: String,

    /// Time of the bundled annotation being compiled on the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_server_ts: Option<MilliSecondsSinceUnixEpoch>,

    /// Number of annotations.
    pub count: UInt,
}

#[cfg(feature = "unstable-msc2677")]
impl BundledAnnotation {
    /// Creates a new `BundledAnnotation` with the given type, key and count.
    pub fn new(annotation_type: AnnotationType, key: String, count: UInt) -> Self {
        Self { annotation_type, key, count, origin_server_ts: None }
    }

    /// Creates a new `BundledAnnotation` for a reaction with the given key and count.
    pub fn reaction(key: String, count: UInt) -> Self {
        Self::new(AnnotationType::Reaction, key, count)
    }
}

/// Type of annotation.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[cfg(feature = "unstable-msc2677")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum AnnotationType {
    /// A reaction.
    #[ruma_enum(rename = "m.reaction")]
    Reaction,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
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
    pub event_id: OwnedEventId,

    /// The user ID of the sender of the latest replacement.
    pub sender: OwnedUserId,

    /// Timestamp in milliseconds on originating homeserver when the latest replacement was sent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_server_ts: Option<MilliSecondsSinceUnixEpoch>,
}

#[cfg(feature = "unstable-msc2676")]
impl BundledReplacement {
    /// Creates a new `BundledReplacement` with the given event ID and sender.
    pub fn new(event_id: OwnedEventId, sender: OwnedUserId) -> Self {
        Self { event_id, sender, origin_server_ts: None }
    }
}

/// A bundled thread.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct BundledThread {
    /// The latest event in the thread.
    pub latest_event: Box<Raw<AnyMessageLikeEvent>>,

    /// The number of events in the thread.
    pub count: UInt,

    /// Whether the current logged in user has participated in the thread.
    pub current_user_participated: bool,
}

impl BundledThread {
    /// Creates a new `BundledThread` with the given event, count and user participated flag.
    pub fn new(
        latest_event: Box<Raw<AnyMessageLikeEvent>>,
        count: UInt,
        current_user_participated: bool,
    ) -> Self {
        Self { latest_event, count, current_user_participated }
    }
}

/// [Bundled aggregations] of related child events.
///
/// [Bundled aggregations]: https://spec.matrix.org/v1.3/client-server-api/#aggregations
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

    /// Thread relation.
    #[serde(rename = "m.thread")]
    pub thread: Option<BundledThread>,
}

impl Relations {
    /// Creates a new empty `Relations`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Relation types as defined in `rel_type` of an `m.relates_to` field.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[non_exhaustive]
pub enum RelationType {
    /// `m.annotation`, an annotation, principally used by reactions.
    #[cfg(feature = "unstable-msc2677")]
    #[ruma_enum(rename = "m.annotation")]
    Annotation,

    /// `m.replace`, a replacement.
    #[cfg(feature = "unstable-msc2676")]
    #[ruma_enum(rename = "m.replace")]
    Replacement,

    /// `m.thread`, a participant to a thread.
    #[ruma_enum(rename = "m.thread")]
    Thread,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
