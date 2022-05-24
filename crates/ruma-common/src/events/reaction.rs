//! Types for the `m.reaction` event.

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::OwnedEventId;

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

/// Information about an annotation relation.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "rel_type", rename = "m.annotation")]
pub struct Relation {
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

impl Relation {
    /// Creates a new `Relation` with the given event ID and key.
    pub fn new(event_id: OwnedEventId, key: String) -> Self {
        Self { event_id, key }
    }
}

#[cfg(test)]
mod tests {
    use crate::event_id;
    use assert_matches::assert_matches;
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
            ReactionEventContent { relates_to: Relation { event_id, key } }
            if event_id == ev_id && key == "🦛"
        );
    }
}
