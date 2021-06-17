//! Types for event relationships.
//!
//! Events in Matrix can relate to one another in a couple of ways, this module adds types to parse
//! the relationship of an event if any exists.
//!
//! MSC for all the relates_to types except replies:
//! <https://github.com/matrix-org/matrix-doc/pull/2674>

use ruma_identifiers::EventId;
use serde::{Deserialize, Serialize};

#[cfg(feature = "unstable-pre-spec")]
use crate::room::message::MessageEventContent;

pub(crate) mod relation_serde;

/// Enum modeling the different ways relationships can be expressed in a `m.relates_to` field of an
/// `m.room.message` or `m.room.encrypted` event.
#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum Relation {
    /// A reference to another event.
    #[cfg(feature = "unstable-pre-spec")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
    Reference(Reference),

    /// An annotation to an event.
    #[cfg(feature = "unstable-pre-spec")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
    Annotation(Annotation),

    /// An event that replaces another event.
    #[cfg(feature = "unstable-pre-spec")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
    Replacement(Replacement),

    /// An `m.in_reply_to` relation indicating that the event is a reply to another event.
    Reply {
        /// Information about another message being replied to.
        in_reply_to: InReplyTo,
    },
}

/// The event this relation belongs to replaces another event.
#[derive(Clone, Debug)]
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
pub struct Replacement {
    /// The ID of the event being replacing.
    pub event_id: EventId,

    /// New content.
    pub new_content: Box<MessageEventContent>,
}

#[cfg(feature = "unstable-pre-spec")]
impl Replacement {
    /// Creates a new `Replacement` with the given event ID and new content.
    pub fn new(event_id: EventId, new_content: Box<MessageEventContent>) -> Self {
        Self { event_id, new_content }
    }
}

/// Information about the event a "rich reply" is replying to.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct InReplyTo {
    /// The event being replied to.
    pub event_id: EventId,
}

impl InReplyTo {
    /// Creates a new `InReplyTo` with the given event ID.
    pub fn new(event_id: EventId) -> Self {
        Self { event_id }
    }
}

/// A reference to another event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Reference {
    /// The event we are referencing.
    pub event_id: EventId,
}

#[cfg(feature = "unstable-pre-spec")]
impl Reference {
    /// Creates a new `Reference` with the given event ID.
    pub fn new(event_id: EventId) -> Self {
        Self { event_id }
    }
}

/// An annotation for an event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Annotation {
    /// The event that is being annotated.
    pub event_id: EventId,

    /// The annotation.
    pub key: String,
}

impl Annotation {
    /// Creates a new `Annotation` with the given event ID and key.
    pub fn new(event_id: EventId, key: String) -> Self {
        Self { event_id, key }
    }
}
