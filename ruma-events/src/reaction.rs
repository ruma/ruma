//! Types for the *m.reaction* event.

use std::convert::TryFrom;

use crate::{
    room::relationships::{Annotation, RelatesToJsonRepr, RelationJsonRepr},
    MessageEvent,
};
use ruma_events_macros::MessageEventContent;
use ruma_identifiers::EventId;
use serde::{Deserialize, Serialize};

/// A reaction to another event.
pub type ReactionEvent = MessageEvent<ReactionEventContent>;

/// The payload for a `ReactionEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, MessageEventContent)]
#[ruma_event(type = "m.reaction")]
pub struct ReactionEventContent {
    /// Information about the related event.
    #[serde(rename = "m.relates_to")]
    pub relation: Relation,
}

/// The relation that contains info which event the reaction is applying to.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "RelatesToJsonRepr", into = "RelatesToJsonRepr")]
pub struct Relation {
    /// The event that is being reacted to.
    pub event_id: EventId,

    /// A string that holds the emoji reaction.
    pub emoji: String,
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

impl ReactionEventContent {
    /// Create a new reaction.
    ///
    /// # Arguments
    ///
    /// * `event_id` - The id of the event we are reacting to.
    /// * `emoji` - The emoji that indicates the reaction that is being applied.
    pub fn new(event_id: EventId, emoji: String) -> Self {
        ReactionEventContent { relation: Relation { event_id, emoji } }
    }
}
