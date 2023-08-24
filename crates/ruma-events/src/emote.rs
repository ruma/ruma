//! Types for extensible emote message events ([MSC3954]).
//!
//! [MSC3954]: https://github.com/matrix-org/matrix-spec-proposals/pull/3954

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{message::TextContentBlock, room::message::Relation};

/// The payload for an extensible emote message.
///
/// This is the new primary type introduced in [MSC3954] and should only be sent in rooms with a
/// version that supports it. See the documentation of the [`message`] module for more information.
///
/// To construct an `EmoteEventContent` with a custom [`TextContentBlock`], convert it with
/// `EmoteEventContent::from()` / `.into()`.
///
/// [MSC3954]: https://github.com/matrix-org/matrix-spec-proposals/pull/3954
/// [`message`]: super::message
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc1767.emote", kind = MessageLike, without_relation)]
pub struct EmoteEventContent {
    /// The message's text content.
    #[serde(rename = "org.matrix.msc1767.text")]
    pub text: TextContentBlock,

    /// Whether this message is automated.
    #[cfg(feature = "unstable-msc3955")]
    #[serde(
        default,
        skip_serializing_if = "ruma_common::serde::is_default",
        rename = "org.matrix.msc1767.automated"
    )]
    pub automated: bool,

    /// Information about related messages.
    #[serde(
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::room::message::relation_serde::deserialize_relation"
    )]
    pub relates_to: Option<Relation<EmoteEventContentWithoutRelation>>,
}

impl EmoteEventContent {
    /// A convenience constructor to create a plain text emote.
    pub fn plain(body: impl Into<String>) -> Self {
        Self {
            text: TextContentBlock::plain(body),
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
            relates_to: None,
        }
    }

    /// A convenience constructor to create an HTML emote.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self {
            text: TextContentBlock::html(body, html_body),
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
            relates_to: None,
        }
    }

    /// A convenience constructor to create an emote from Markdown.
    ///
    /// The content includes an HTML message if some Markdown formatting was detected, otherwise
    /// only a plain text message is included.
    #[cfg(feature = "markdown")]
    pub fn markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self {
            text: TextContentBlock::markdown(body),
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
            relates_to: None,
        }
    }
}

impl From<TextContentBlock> for EmoteEventContent {
    fn from(text: TextContentBlock) -> Self {
        Self {
            text,
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
            relates_to: None,
        }
    }
}
