//! Types for the *m.tag* event.

use std::collections::HashMap;

use ruma_events_macros::ruma_event;
use serde::{Deserialize, Serialize};

ruma_event! {
    /// Informs the client of tags on a room.
    TagEvent {
        kind: Event,
        event_type: Tag,
        content: {
            /// A map of tag names to tag info.
            pub tags: HashMap<String, TagInfo>,
        },
    }
}

/// Information about a tag.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TagInfo {
    /// Value to use for lexicographically ordering rooms with this tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<f64>,
}
