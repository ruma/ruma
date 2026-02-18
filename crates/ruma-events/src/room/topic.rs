//! Types for the [`m.room.topic`] event.
//!
//! [`m.room.topic`]: https://spec.matrix.org/latest/client-server-api/#mroomtopic

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{EmptyStateKey, message::TextContentBlock};

/// The content of an `m.room.topic` event.
///
/// A topic is a short message detailing what is currently being discussed in the room.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.room.topic", kind = State, state_key_type = EmptyStateKey)]
pub struct RoomTopicEventContent {
    /// The topic as plain text.
    ///
    /// This SHOULD duplicate the content of the `text/plain` representation in `topic_block` if
    /// any exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,

    /// Textual representation of the room topic in different mimetypes.
    ///
    /// With the `compat-lax-room-topic-deser` cargo feature, this field is ignored if its
    /// deserialization fails.
    #[serde(rename = "m.topic", default, skip_serializing_if = "TopicContentBlock::is_empty")]
    #[cfg_attr(
        feature = "compat-lax-room-topic-deser",
        serde(deserialize_with = "ruma_common::serde::default_on_error")
    )]
    pub topic_block: TopicContentBlock,
}

impl RoomTopicEventContent {
    /// Creates a new `RoomTopicEventContent` with the given plain text topic.
    pub fn new(topic: String) -> Self {
        Self { topic_block: TopicContentBlock::plain(topic.clone()), topic: Some(topic) }
    }

    /// Convenience constructor to create a new HTML topic with a plain text fallback.
    pub fn html(plain: impl Into<String>, html: impl Into<String>) -> Self {
        let plain = plain.into();
        Self { topic: Some(plain.clone()), topic_block: TopicContentBlock::html(plain, html) }
    }

    /// Convenience constructor to create a topic from Markdown.
    ///
    /// The content includes an HTML topic if some Markdown formatting was detected, otherwise
    /// only a plain text topic is included.
    #[cfg(feature = "markdown")]
    pub fn markdown(topic: impl AsRef<str> + Into<String>) -> Self {
        let plain = topic.as_ref().to_owned();
        Self { topic: Some(plain), topic_block: TopicContentBlock::markdown(topic) }
    }
}

/// A block for topic content.
///
/// To construct a `TopicContentBlock` with a custom [`TextContentBlock`], convert it with
/// `TopicContentBlock::from()` / `.into()`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct TopicContentBlock {
    /// The text representations of the topic.
    #[serde(rename = "m.text")]
    pub text: TextContentBlock,
}

impl TopicContentBlock {
    /// A convenience constructor to create a plain text `TopicContentBlock`.
    pub fn plain(body: impl Into<String>) -> Self {
        Self { text: TextContentBlock::plain(body) }
    }

    /// A convenience constructor to create an HTML `TopicContentBlock`.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self { text: TextContentBlock::html(body, html_body) }
    }

    /// A convenience constructor to create a `TopicContentBlock` from Markdown.
    ///
    /// The content includes an HTML topic if some Markdown formatting was detected, otherwise
    /// only a plain text topic is included.
    #[cfg(feature = "markdown")]
    pub fn markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self { text: TextContentBlock::markdown(body) }
    }

    /// Whether this content block is empty.
    fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

impl From<TextContentBlock> for TopicContentBlock {
    fn from(text: TextContentBlock) -> Self {
        Self { text }
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::canonical_json::assert_to_canonical_json_eq;
    use serde_json::{from_value as from_json_value, json};

    use super::RoomTopicEventContent;
    use crate::message::TextContentBlock;

    #[test]
    fn serialize_content() {
        // Content with plain text block.
        let mut content = RoomTopicEventContent::new("Hot Topic".to_owned());
        assert_to_canonical_json_eq!(
            content,
            json!({
                "topic": "Hot Topic",
                "m.topic": {
                   "m.text": [
                        { "body": "Hot Topic" },
                    ],
                }
            })
        );

        // Content without block.
        content.topic_block.text = TextContentBlock::from(vec![]);
        assert_to_canonical_json_eq!(
            content,
            json!({
                "topic": "Hot Topic",
            })
        );

        // Content with HTML block.
        let content = RoomTopicEventContent::html("Hot Topic", "<strong>Hot</strong> Topic");
        assert_to_canonical_json_eq!(
            content,
            json!({
                "topic": "Hot Topic",
                "m.topic": {
                   "m.text": [
                        { "body": "<strong>Hot</strong> Topic", "mimetype": "text/html" },
                        { "body": "Hot Topic" },
                    ],
                }
            })
        );
    }

    #[test]
    fn deserialize_content() {
        let json = json!({
            "topic": "Hot Topic",
            "m.topic": {
               "m.text": [
                    { "body": "<strong>Hot</strong> Topic", "mimetype": "text/html" },
                    { "body": "Hot Topic" },
                ],
            }
        });

        let content = from_json_value::<RoomTopicEventContent>(json).unwrap();
        assert_eq!(content.topic.as_deref(), Some("Hot Topic"));
        assert_eq!(content.topic_block.text.find_html(), Some("<strong>Hot</strong> Topic"));
        assert_eq!(content.topic_block.text.find_plain(), Some("Hot Topic"));

        let content = serde_json::from_str::<RoomTopicEventContent>(
            r#"{"topic":"Hot Topic","m.topic":{"m.text":[{"body":"Hot Topic"}]}}"#,
        )
        .unwrap();
        assert_eq!(content.topic.as_deref(), Some("Hot Topic"));
        assert_eq!(content.topic_block.text.find_html(), None);
        assert_eq!(content.topic_block.text.find_plain(), Some("Hot Topic"));
    }

    #[test]
    fn deserialize_event() {
        let json = json!({
            "content": {
                "topic": "Hot Topic",
                "m.topic": {
                    "m.text": [
                        { "body": "<strong>Hot</strong> Topic", "mimetype": "text/html" },
                        { "body": "Hot Topic" },
                    ],
                },
            },
            "type": "m.room.topic",
            "state_key": "",
            "event_id": "$lkioKdioukshnlDDz",
            "sender": "@alice:localhost",
            "origin_server_ts": 309_998_934,
        });

        from_json_value::<super::SyncRoomTopicEvent>(json).unwrap();
    }

    #[test]
    #[cfg(feature = "compat-lax-room-topic-deser")]
    fn deserialize_invalid_content() {
        let json = json!({
            "topic": "Hot Topic",
            "m.topic": [
                { "body": "<strong>Hot</strong> Topic", "mimetype": "text/html" },
                { "body": "Hot Topic" },
            ],
        });

        let content = from_json_value::<RoomTopicEventContent>(json).unwrap();
        assert_eq!(content.topic.as_deref(), Some("Hot Topic"));
        assert_eq!(content.topic_block.text.find_html(), None);
        assert_eq!(content.topic_block.text.find_plain(), None);

        let content = serde_json::from_str::<RoomTopicEventContent>(
            r#"{"topic":"Hot Topic","m.topic":[{"body":"Hot Topic"}]}"#,
        )
        .unwrap();
        assert_eq!(content.topic.as_deref(), Some("Hot Topic"));
        assert_eq!(content.topic_block.text.find_html(), None);
        assert_eq!(content.topic_block.text.find_plain(), None);
    }

    #[test]
    #[cfg(feature = "compat-lax-room-topic-deser")]
    fn deserialize_invalid_event() {
        let json = json!({
            "content": {
                "topic": "Hot Topic",
                "m.topic": [
                    { "body": "<strong>Hot</strong> Topic", "mimetype": "text/html" },
                    { "body": "Hot Topic" },
                ],
            },
            "type": "m.room.topic",
            "state_key": "",
            "event_id": "$lkioKdioukshnlDDz",
            "sender": "@alice:localhost",
            "origin_server_ts": 309_998_934,
        });

        from_json_value::<super::SyncRoomTopicEvent>(json).unwrap();
    }
}
