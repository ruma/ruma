use serde::{Deserialize, Serialize};

#[cfg(feature = "unstable-msc1767")]
use crate::events::message::MessageContent;

use super::FormattedBody;

/// The payload for a text message.
///
/// With the `unstable-msc1767` feature, this type contains the transitional format of
/// [`MessageEventContent`]. See the documentation of the [`message`] module for more information.
///
/// [`MessageEventContent`]: crate::events::message::MessageEventContent
/// [`message`]: crate::events::message
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.text")]
pub struct TextMessageEventContent {
    /// The body of the message.
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

impl TextMessageEventContent {
    /// A convenience constructor to create a plain text message.
    pub fn plain(body: impl Into<String>) -> Self {
        let body = body.into();
        Self {
            #[cfg(feature = "unstable-msc1767")]
            message: Some(MessageContent::plain(body.clone())),
            body,
            formatted: None,
        }
    }

    /// A convenience constructor to create an HTML message.
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

    /// A convenience constructor to create a Markdown message.
    ///
    /// Returns an HTML message if some Markdown formatting was detected, otherwise returns a plain
    /// text message.
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
impl From<MessageContent> for TextMessageEventContent {
    fn from(message: MessageContent) -> Self {
        let body = if let Some(body) = message.find_plain() { body } else { &message[0].body };
        let formatted = message.find_html().map(FormattedBody::html);

        Self { body: body.to_owned(), formatted, message: Some(message) }
    }
}
