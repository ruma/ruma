//! Types for the *m.tag* event.

use std::collections::HashMap;

use events::EventType;

/// Informs the client of tags on a room.
#[derive(Debug, Deserialize, Serialize)]
pub struct TagEvent {
    /// The payload.
    pub content: TagEventContent,
    pub event_type: EventType,
}

/// The payload of a `TagEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct TagEventContent {
    /// A map of tag names to tag info.
    pub tags: HashMap<String, TagInfo>,
}

/// Information about a tag.
#[derive(Debug, Deserialize, Serialize)]
pub struct TagInfo {
    pub order: Option<u64>,
}
