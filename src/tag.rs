//! Types for the *m.tag* event.

use std::collections::HashMap;

use Event;

/// Informs the client of tags on a room.
pub type TagEvent = Event<TagEventContent, ()>;

/// The payload of a `TagEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct TagEventContent {
    /// A map of tag names to tag info.
    pub tags: HashMap<String, TagInfo>,
}

/// Information about a tag.
#[derive(Debug, Deserialize, Serialize)]
pub struct TagInfo {
    /// Value to use for lexicographically ordering rooms with this tag.
    #[serde(skip_serializing_if="Option::is_none")]
    pub order: Option<u64>,
}
