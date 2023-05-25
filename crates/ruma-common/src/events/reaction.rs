//! Types for the [`m.reaction`] event.
//!
//! [`m.reaction`]: https://spec.matrix.org/latest/client-server-api/#mreaction

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
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::ReactionEventContent;
    use crate::{event_id, events::relation::Annotation};

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

    #[test]
    fn serialize() {
        let content = ReactionEventContent::new(Annotation::new(
            event_id!("$my_reaction").to_owned(),
            "🏠".to_owned(),
        ));

        assert_eq!(
            to_json_value(&content).unwrap(),
            json!({
                "m.relates_to": {
                    "rel_type": "m.annotation",
                    "event_id": "$my_reaction",
                    "key": "🏠"
                }
            })
        );
    }
}
