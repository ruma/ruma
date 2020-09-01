//! Types for the *m.tag* event.

use std::collections::BTreeMap;

use ruma_events_macros::BasicEventContent;
use serde::{Deserialize, Serialize};

use crate::BasicEvent;

/// Informs the client of tags on a room.
pub type TagEvent = BasicEvent<TagEventContent>;

/// Map of tag names to tag info.
pub type Tags = BTreeMap<String, TagInfo>;

/// The payload for `TagEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, BasicEventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.tag")]
pub struct TagEventContent {
    /// A map of tag names to tag info.
    pub tags: Tags,
}

impl TagEventContent {
    /// Creates a new `TagEventContent` with the given `Tags`.
    pub fn new(tags: Tags) -> Self {
        Self { tags }
    }
}

impl From<Tags> for TagEventContent {
    fn from(tags: Tags) -> Self {
        Self::new(tags)
    }
}

/// Information about a tag.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct TagInfo {
    /// Value to use for lexicographically ordering rooms with this tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<f64>,
}

impl TagInfo {
    /// Creates an empty `TagInfo`.
    pub fn new() -> Self {
        Default::default()
    }
}
