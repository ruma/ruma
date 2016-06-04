//! Types for the *m.tag* event.

use std::collections::HashMap;

use core::EventType;

/// Informs the client of tags on a room.
pub struct TagEvent {
    /// The payload.
    content: TagEventContent,
}

/// The payload of a `TagEvent`.
pub struct TagEventContent {
    /// A map of tag names to tag info.
    tags: HashMap<String, TagInfo>,
}

/// Information about a tag.
pub struct TagInfo {
    order: Option<u64>,
}
