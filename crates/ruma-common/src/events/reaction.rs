//! Types for the `m.reaction` event.

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::relation::Annotation;

/// The payload for a `m.reaction` event.
///
/// A reaction to another event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.reaction", kind = MessageLike)]
pub struct ReactionEventContent {
    /// Information about the related event.
    #[serde(rename = "m.relates_to")]
    pub relates_to: Annotation,
}

impl ReactionEventContent {
    /// Creates a new `ReactionEventContent` from the given annotation.
    ///
    /// You can also construct a `ReactionEventContent` from an annotation using `From` / `Into`.
    pub fn new(relates_to: Annotation) -> Self {
        Self { relates_to }
    }
}

impl From<Annotation> for ReactionEventContent {
    fn from(relates_to: Annotation) -> Self {
        Self::new(relates_to)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use serde_json::{from_value as from_json_value, json};

    use super::ReactionEventContent;

    #[test]
    fn deserialize() {
        let json = json!({
            "m.relates_to": {
                "rel_type": "m.annotation",
                "event_id": "$1598361704261elfgc:localhost",
                "key": "🦛",
            }
        });

        let relates_to = assert_matches!(
            from_json_value::<ReactionEventContent>(json),
            Ok(ReactionEventContent { relates_to }) => relates_to
        );
        assert_eq!(relates_to.event_id, "$1598361704261elfgc:localhost");
        assert_eq!(relates_to.key, "🦛");
    }
}
