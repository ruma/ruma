//! Types for the `m.reaction` event.

use ruma_events_macros::EventContent;
use ruma_identifiers::EventId;
use serde::{Deserialize, Serialize};

/// The payload for a `m.reaction` event.
///
/// A reaction to another event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.reaction", kind = MessageLike)]
pub struct ReactionEventContent {
    /// Information about the related event.
    #[serde(rename = "m.relates_to")]
    pub relates_to: Relation,
}

impl ReactionEventContent {
    /// Creates a new `ReactionEventContent` from the given relation.
    ///
    /// You can also construct a `ReactionEventContent` from a relation using `From` / `Into`.
    pub fn new(relates_to: Relation) -> Self {
        Self { relates_to }
    }
}

impl From<Relation> for ReactionEventContent {
    fn from(relates_to: Relation) -> Self {
        Self::new(relates_to)
    }
}

/// The relation that contains info which event the reaction is applying to.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "rel_type", rename = "m.annotation")]
pub struct Relation {
    /// The event that is being reacted to.
    pub event_id: Box<EventId>,

    /// A string that holds the emoji reaction.
    #[serde(rename = "key")]
    pub emoji: String,
}

impl Relation {
    /// Creates a new `Relation` with the given event ID and emoji.
    pub fn new(event_id: Box<EventId>, emoji: String) -> Self {
        Self { event_id, emoji }
    }
}

#[cfg(test)]
mod tests {
    use matches::assert_matches;
    use ruma_identifiers::event_id;
    use serde_json::{from_value as from_json_value, json};

    use super::{ReactionEventContent, Relation};

    #[test]
    fn deserialize() {
        let ev_id = event_id!("$1598361704261elfgc:localhost");

        let json = json!({
            "m.relates_to": {
                "rel_type": "m.annotation",
                "event_id": ev_id,
                "key": "🦛",
            }
        });

        assert_matches!(
            from_json_value::<ReactionEventContent>(json).unwrap(),
            ReactionEventContent { relates_to: Relation { event_id, emoji } }
            if event_id == ev_id && emoji == "🦛"
        );
    }
}
