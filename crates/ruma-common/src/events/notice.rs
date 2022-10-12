//! Types for extensible notice message events ([MSC1767]).
//!
//! [MSC1767]: https://github.com/matrix-org/matrix-spec-proposals/pull/1767

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{message::MessageContent, room::message::Relation};

/// The payload for an extensible notice message.
///
/// This is the new primary type introduced in [MSC1767] and should not be sent before the end of
/// the transition period. See the documentation of the [`message`] module for more information.
///
/// [MSC1767]: https://github.com/matrix-org/matrix-spec-proposals/pull/1767
/// [`message`]: super::message
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.notice", kind = MessageLike, without_relation)]
pub struct NoticeEventContent {
    /// The message's text content.
    #[serde(flatten)]
    pub message: MessageContent,

    /// Information about related messages.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Relation>,
}

impl NoticeEventContent {
    /// A convenience constructor to create a plain text notice.
    pub fn plain(body: impl Into<String>) -> Self {
        Self { message: MessageContent::plain(body), relates_to: None }
    }

    /// A convenience constructor to create an HTML notice.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self { message: MessageContent::html(body, html_body), relates_to: None }
    }

    /// A convenience constructor to create a Markdown notice.
    ///
    /// Returns an HTML notice if some Markdown formatting was detected, otherwise returns a plain
    /// text notice.
    #[cfg(feature = "markdown")]
    pub fn markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self { message: MessageContent::markdown(body), relates_to: None }
    }
}
