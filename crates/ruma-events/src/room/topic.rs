//! Types for the [`m.room.topic`] event.
//!
//! [`m.room.topic`]: https://spec.matrix.org/latest/client-server-api/#mroomtopic

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

#[cfg(feature = "unstable-msc3765")]
use crate::message::TextContentBlock;
use crate::EmptyStateKey;

/// The content of an `m.room.topic` event.
///
/// A topic is a short message detailing what is currently being discussed in the room.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.room.topic", kind = State, state_key_type = EmptyStateKey)]
pub struct RoomTopicEventContent {
    /// The topic as plain text.
    ///
    /// If the `unstable-msc3765` cargo feature is enabled, this SHOULD duplicate the content of
    /// the `text/plain` representation in `topic_block` if any exists.
    pub topic: String,

    /// Textual representation of the room topic in different mimetypes ([MSC3765]).
    ///
    /// Uses the unstable prefix `org.matrix.msc3765.topic`.
    ///
    /// [MSC3765]: https://github.com/matrix-org/matrix-spec-proposals/pull/3765
    #[cfg(feature = "unstable-msc3765")]
    #[serde(
        rename = "org.matrix.msc3765.topic",
        default,
        skip_serializing_if = "TextContentBlock::is_empty"
    )]
    pub topic_block: TextContentBlock,
}

impl RoomTopicEventContent {
    /// Creates a new `RoomTopicEventContent` with the given plain text topic.
    pub fn new(topic: String) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3765")]
            topic_block: TextContentBlock::plain(topic.clone()),
            topic,
        }
    }

    /// Convenience constructor to create a new HTML topic with a plain text fallback.
    #[cfg(feature = "unstable-msc3765")]
    pub fn html(plain: impl Into<String>, html: impl Into<String>) -> Self {
        let plain = plain.into();
        Self { topic: plain.clone(), topic_block: TextContentBlock::html(plain, html) }
    }

    /// Convenience constructor to create a topic from Markdown.
    ///
    /// The content includes an HTML topic if some Markdown formatting was detected, otherwise
    /// only a plain text topic is included.
    #[cfg(all(feature = "unstable-msc3765", feature = "markdown"))]
    pub fn markdown(topic: impl AsRef<str> + Into<String>) -> Self {
        let plain = topic.as_ref().to_owned();
        Self { topic: plain, topic_block: TextContentBlock::markdown(topic) }
    }
}
