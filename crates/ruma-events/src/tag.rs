//! Types for the *m.tag* event.

use std::collections::BTreeMap;

use ruma_events_macros::BasicEventContent;
use ruma_serde::StringEnum;
use serde::{Deserialize, Serialize};

use crate::BasicEvent;

/// Informs the client of tags on a room.
pub type TagEvent = BasicEvent<TagEventContent>;

/// Map of tag names to tag info.
pub type Tags = BTreeMap<TagName, TagInfo>;

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

/// The name of a tag.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, StringEnum)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum TagName {
    /// `m.favourite`: The user's favourite rooms. These should be shown with higher precedence
    /// than other rooms.
    #[ruma_enum(rename = "m.favourite")]
    Favorite,

    /// `m.lowpriority`: These should be shown with lower precedence than others.
    #[ruma_enum(rename = "m.lowpriority")]
    LowPriority,

    /// `m.server_notice`: Used to identify
    /// [Server Notice Rooms](https://matrix.org/docs/spec/client_server/r0.6.1#module-server-notices).
    #[ruma_enum(rename = "m.server_notice")]
    ServerNotice,

    /// A custom tag
    #[doc(hidden)]
    _Custom(String),
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

#[cfg(test)]
mod tests {
    use maplit::btreemap;
    use serde_json::{json, to_value as to_json_value};

    use super::{TagEventContent, TagInfo, TagName};

    #[test]
    fn serialization() {
        let tags = btreemap! {
            TagName::Favorite => TagInfo::new(),
            TagName::LowPriority => TagInfo::new(),
            TagName::ServerNotice => TagInfo::new(),
            "u.custom".to_owned().into() => TagInfo { order: Some(0.9) }
        };

        let content = TagEventContent { tags };

        assert_eq!(
            to_json_value(content).unwrap(),
            json!({
                "tags": {
                    "m.favourite": {},
                    "m.lowpriority": {},
                    "m.server_notice": {},
                    "u.custom": {
                        "order": 0.9
                    }
                },
            })
        );
    }
}
