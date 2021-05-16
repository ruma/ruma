//! Types for the *m.reaction* event.

use std::convert::TryFrom;

use ruma_events_macros::EventContent;
use ruma_identifiers::EventId;
use serde::{Deserialize, Serialize};

use crate::{
    room::relationships::{Annotation, RelatesToJsonRepr, RelationJsonRepr},
    MessageEvent,
};

/// A reaction to another event.
pub type ReactionEvent = MessageEvent<ReactionEventContent>;

/// The payload for a `ReactionEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.reaction", kind = Message)]
pub struct ReactionEventContent {
    /// Information about the related event.
    #[serde(rename = "m.relates_to")]
    pub relation: Relation,
}

impl ReactionEventContent {
    /// Creates a new `ReactionEventContent` from the given relation.
    ///
    /// You can also construct a `ReactionEventContent` from a relation using `From` / `Into`.
    pub fn new(relation: Relation) -> Self {
        Self { relation }
    }
}

impl From<Relation> for ReactionEventContent {
    fn from(relation: Relation) -> Self {
        Self::new(relation)
    }
}

/// The relation that contains info which event the reaction is applying to.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(try_from = "RelatesToJsonRepr", into = "RelatesToJsonRepr")]
pub struct Relation {
    /// The event that is being reacted to.
    pub event_id: EventId,

    /// A string that holds the emoji reaction.
    pub emoji: String,
}

impl Relation {
    /// Creates a new `Relation` with the given event ID and emoji.
    pub fn new(event_id: EventId, emoji: String) -> Self {
        Self { event_id, emoji }
    }
}

impl From<Relation> for RelatesToJsonRepr {
    fn from(relation: Relation) -> Self {
        RelatesToJsonRepr::Relation(RelationJsonRepr::Annotation(Annotation {
            event_id: relation.event_id,
            key: relation.emoji,
        }))
    }
}

impl TryFrom<RelatesToJsonRepr> for Relation {
    type Error = &'static str;

    fn try_from(value: RelatesToJsonRepr) -> Result<Self, Self::Error> {
        if let RelatesToJsonRepr::Relation(RelationJsonRepr::Annotation(a)) = value {
            Ok(Relation { event_id: a.event_id, emoji: a.key })
        } else {
            Err("Expected a relation with a rel_type of `annotation`")
        }
    }
}
