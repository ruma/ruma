//! Types describing [relationships between events].
//!
//! [relationships between events]: https://spec.matrix.org/v1.4/client-server-api/#forming-relationships-between-events

use std::fmt::Debug;

use js_int::UInt;
use serde::{Deserialize, Serialize};

use super::AnyMessageLikeEvent;
use crate::{
    serde::{Raw, StringEnum},
    MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedUserId, PrivOwnedStr,
};

/// Information about the event a [rich reply] is replying to.
///
/// [rich reply]: https://spec.matrix.org/v1.5/client-server-api/#rich-replies
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct InReplyTo {
    /// The event being replied to.
    pub event_id: OwnedEventId,
}

impl InReplyTo {
    /// Creates a new `InReplyTo` with the given event ID.
    pub fn new(event_id: OwnedEventId) -> Self {
        Self { event_id }
    }
}

/// An [annotation] for an event.
///
/// [annotation]: https://github.com/matrix-org/matrix-spec-proposals/pull/2677
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg(feature = "unstable-msc2677")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "rel_type", rename = "m.annotation")]
pub struct Annotation {
    /// The event that is being annotated.
    pub event_id: OwnedEventId,

    /// A string that indicates the annotation being applied.
    ///
    /// When sending emoji reactions, this field should include the colourful variation-16 when
    /// applicable.
    ///
    /// Clients should render reactions that have a long `key` field in a sensible manner.
    pub key: String,
}

#[cfg(feature = "unstable-msc2677")]
impl Annotation {
    /// Creates a new `Annotation` with the given event ID and key.
    pub fn new(event_id: OwnedEventId, key: String) -> Self {
        Self { event_id, key }
    }
}

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
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct BundledReplacement {
    /// The ID of the replacing event.
    pub event_id: OwnedEventId,

    /// The user ID of the sender of the latest replacement.
    pub sender: OwnedUserId,

    /// Timestamp in milliseconds on originating homeserver when the latest replacement was sent.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,
}

impl BundledReplacement {
    /// Creates a new `BundledReplacement` with the given event ID, sender and timestamp.
    pub fn new(
        event_id: OwnedEventId,
        sender: OwnedUserId,
        origin_server_ts: MilliSecondsSinceUnixEpoch,
    ) -> Self {
        Self { event_id, sender, origin_server_ts }
    }
}

/// The content of a [replacement] relation.
///
/// [replacement]: https://spec.matrix.org/v1.5/client-server-api/#event-replacements
#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Replacement<C> {
    /// The ID of the event being replaced.
    pub event_id: OwnedEventId,

    /// New content.
    pub new_content: C,
}

impl<C> Replacement<C> {
    /// Creates a new `Replacement` with the given event ID and new content.
    pub fn new(event_id: OwnedEventId, new_content: C) -> Self {
        Self { event_id, new_content }
    }
}

/// The content of a [thread] relation.
///
/// [thread]: https://spec.matrix.org/v1.5/client-server-api/#threading
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Thread {
    /// The ID of the root message in the thread.
    pub event_id: OwnedEventId,

    /// A reply relation.
    ///
    /// If this event is a reply and belongs to a thread, this points to the message that is being
    /// replied to, and `is_falling_back` must be set to `false`.
    ///
    /// If this event is not a reply, this is used as a fallback mechanism for clients that do not
    /// support threads. This should point to the latest message-like event in the thread and
    /// `is_falling_back` must be set to `true`.
    pub in_reply_to: InReplyTo,

    /// Whether the `m.in_reply_to` field is a fallback for older clients or a genuine reply in a
    /// thread.
    pub is_falling_back: bool,
}

impl Thread {
    /// Convenience method to create a regular `Thread` with the given event ID and latest
    /// message-like event ID.
    pub fn plain(event_id: OwnedEventId, latest_event_id: OwnedEventId) -> Self {
        Self { event_id, in_reply_to: InReplyTo::new(latest_event_id), is_falling_back: true }
    }

    /// Convenience method to create a reply `Thread` with the given event ID and replied-to event
    /// ID.
    pub fn reply(event_id: OwnedEventId, reply_to_event_id: OwnedEventId) -> Self {
        Self { event_id, in_reply_to: InReplyTo::new(reply_to_event_id), is_falling_back: false }
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

/// A [reference] to another event.
///
/// [reference]: https://spec.matrix.org/v1.5/client-server-api/#reference-relations
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "rel_type", rename = "m.reference")]
pub struct Reference {
    /// The ID of the event being referenced.
    pub event_id: OwnedEventId,
}

impl Reference {
    /// Creates a new `Reference` with the given event ID.
    pub fn new(event_id: OwnedEventId) -> Self {
        Self { event_id }
    }
}

/// A bundled reference.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct BundledReference {
    /// The ID of the event referencing this event.
    pub event_id: OwnedEventId,
}

impl BundledReference {
    /// Creates a new `BundledThread` with the given event ID.
    pub fn new(event_id: OwnedEventId) -> Self {
        Self { event_id }
    }
}

/// A chunk of references.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ReferenceChunk {
    /// A batch of bundled references.
    pub chunk: Vec<BundledReference>,
}

impl ReferenceChunk {
    /// Creates a new `ReferenceChunk` with the given chunk.
    pub fn new(chunk: Vec<BundledReference>) -> Self {
        Self { chunk }
    }
}

/// [Bundled aggregations] of related child events.
///
/// [Bundled aggregations]: https://spec.matrix.org/v1.4/client-server-api/#aggregations
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct BundledRelations {
    /// Annotation relations.
    #[cfg(feature = "unstable-msc2677")]
    #[serde(rename = "m.annotation")]
    pub annotation: Option<AnnotationChunk>,

    /// Replacement relation.
    #[serde(rename = "m.replace")]
    pub replace: Option<BundledReplacement>,

    /// Thread relation.
    #[serde(rename = "m.thread")]
    pub thread: Option<BundledThread>,

    /// Reference relations.
    #[serde(rename = "m.reference")]
    pub reference: Option<ReferenceChunk>,
}

impl BundledRelations {
    /// Creates a new empty `BundledRelations`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Relation types as defined in `rel_type` of an `m.relates_to` field.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "m.snake_case")]
#[non_exhaustive]
pub enum RelationType {
    /// `m.annotation`, an annotation, principally used by reactions.
    #[cfg(feature = "unstable-msc2677")]
    Annotation,

    /// `m.replace`, a replacement.
    Replacement,

    /// `m.thread`, a participant to a thread.
    Thread,

    /// `m.reference`, a reference to another event.
    Reference,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
