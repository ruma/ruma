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
#[ruma_event(type = "m.tag")]
pub struct TagEventContent {
    /// A map of tag names to tag info.
    pub tags: Tags,
}

/// Information about a tag.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TagInfo {
    /// Value to use for lexicographically ordering rooms with this tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<f64>,
}
