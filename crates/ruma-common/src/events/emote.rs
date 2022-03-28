//! Types for extensible emote message events ([MSC1767]).
//!
//! [MSC1767]: https://github.com/matrix-org/matrix-spec-proposals/pull/1767

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{
    message::MessageContent,
    room::message::{EmoteMessageEventContent, Relation},
};

/// The payload for an extensible emote message.
///
/// This is the new primary type introduced in [MSC1767] and should not be sent before the end of
/// the transition period. See the documentation of the [`message`] module for more information.
///
/// `EmoteEventContent` can be converted to a [`RoomMessageEventContent`] with a
/// [`MessageType::Emote`]. You can convert it back with
/// [`EmoteEventContent::from_emote_room_message()`].
///
/// [MSC1767]: https://github.com/matrix-org/matrix-spec-proposals/pull/1767
/// [`message`]: super::message
/// [`RoomMessageEventContent`]: super::room::message::RoomMessageEventContent
/// [`MessageType::Emote`]: super::room::message::MessageType::Emote
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.emote", kind = MessageLike)]
pub struct EmoteEventContent {
    /// The message's text content.
    #[serde(flatten)]
    pub message: MessageContent,

    /// Information about related messages.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Relation>,
}

impl EmoteEventContent {
    /// A convenience constructor to create a plain text emote.
    pub fn plain(body: impl Into<String>) -> Self {
        Self { message: MessageContent::plain(body), relates_to: None }
    }

    /// A convenience constructor to create an HTML emote.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self { message: MessageContent::html(body, html_body), relates_to: None }
    }

    /// A convenience constructor to create a Markdown emote.
    ///
    /// Returns an HTML emote if some Markdown formatting was detected, otherwise returns a plain
    /// text emote.
    #[cfg(feature = "markdown")]
    pub fn markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self { message: MessageContent::markdown(body), relates_to: None }
    }

    /// Create a new `EmoteEventContent` from the given `EmoteMessageEventContent` and optional
    /// relation.
    pub fn from_emote_room_message(
        content: EmoteMessageEventContent,
        relates_to: Option<Relation>,
    ) -> Self {
        let EmoteMessageEventContent { body, formatted, message, .. } = content;
        if let Some(message) = message {
            Self { message, relates_to }
        } else {
            Self { message: MessageContent::from_room_message_content(body, formatted), relates_to }
        }
    }
}
