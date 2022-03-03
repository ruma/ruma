//! Types for extensible emote message events ([MSC1767]).
//!
//! [MSC1767]: https://github.com/matrix-org/matrix-spec-proposals/pull/1767

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{message::MessageContent, room::message::Relation};

/// The payload for an extensible emote message.
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.emote", kind = MessageLike)]
pub struct EmoteEventContent {
    /// The message's text content.
    #[serde(flatten)]
    pub message: MessageContent,

    /// Information about related messages for [rich replies].
    ///
    /// [rich replies]: https://spec.matrix.org/v1.2/client-server-api/#rich-replies
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Relation>,
}

impl EmoteEventContent {
    /// A convenience constructor to create a plain text message.
    pub fn plain(body: impl Into<String>) -> Self {
        Self { message: MessageContent::plain(body), relates_to: None }
    }

    /// A convenience constructor to create an HTML message.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self { message: MessageContent::html(body, html_body), relates_to: None }
    }

    /// A convenience constructor to create a Markdown message.
    ///
    /// Returns an HTML message if some Markdown formatting was detected, otherwise returns a plain
    /// text message.
    #[cfg(feature = "markdown")]
    pub fn markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self { message: MessageContent::markdown(body), relates_to: None }
    }
}
