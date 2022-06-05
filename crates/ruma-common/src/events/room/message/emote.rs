use serde::{Deserialize, Serialize};

use super::FormattedBody;
#[cfg(feature = "unstable-msc1767")]
use crate::events::message::MessageContent;

/// The payload for an emote message.
///
/// With the `unstable-msc1767` feature, this type contains the transitional format of
/// [`EmoteEventContent`]. See the documentation of the [`message`] module for more information.
///
/// [`EmoteEventContent`]: crate::events::emote::EmoteEventContent
/// [`message`]: crate::events::message
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.emote")]
pub struct EmoteMessageEventContent {
    /// The emote action to perform.
    pub body: String,

    /// Formatted form of the message `body`.
    #[serde(flatten)]
    pub formatted: Option<FormattedBody>,

    /// Extensible-event representation of the message.
    ///
    /// If present, this should be preferred over the other fields.
    #[cfg(feature = "unstable-msc1767")]
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub message: Option<MessageContent>,
}

impl EmoteMessageEventContent {
    /// A convenience constructor to create a plain-text emote.
    pub fn plain(body: impl Into<String>) -> Self {
        let body = body.into();
        Self {
            #[cfg(feature = "unstable-msc1767")]
            message: Some(MessageContent::plain(body.clone())),
            body,
            formatted: None,
        }
    }

    /// A convenience constructor to create an html emote message.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        let body = body.into();
        let html_body = html_body.into();
        Self {
            #[cfg(feature = "unstable-msc1767")]
            message: Some(MessageContent::html(body.clone(), html_body.clone())),
            body,
            formatted: Some(FormattedBody::html(html_body)),
        }
    }

    /// A convenience constructor to create a markdown emote.
    ///
    /// Returns an html emote message if some markdown formatting was detected, otherwise returns a
    /// plain-text emote.
    #[cfg(feature = "markdown")]
    pub fn markdown(body: impl AsRef<str> + Into<String>) -> Self {
        if let Some(formatted) = FormattedBody::markdown(&body) {
            Self::html(body, formatted.body)
        } else {
            Self::plain(body)
        }
    }
}

#[cfg(feature = "unstable-msc1767")]
impl From<MessageContent> for EmoteMessageEventContent {
    fn from(message: MessageContent) -> Self {
        let body = if let Some(body) = message.find_plain() { body } else { &message[0].body };
        let formatted = message.find_html().map(FormattedBody::html);

        Self { body: body.to_owned(), formatted, message: Some(message) }
    }
}
