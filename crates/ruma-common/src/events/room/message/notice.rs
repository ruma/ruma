use serde::{Deserialize, Serialize};

#[cfg(feature = "unstable-msc1767")]
use crate::events::message::MessageContent;

use super::FormattedBody;

/// The payload for a notice message.
///
/// With the `unstable-msc1767` feature, this type contains the transitional format of
/// [`NoticeEventContent`]. See the documentation of the [`message`] module for more information.
///
/// [`NoticeEventContent`]: crate::events::notice::NoticeEventContent
/// [`message`]: crate::events::message
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.notice")]
pub struct NoticeMessageEventContent {
    /// The notice text.
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

impl NoticeMessageEventContent {
    /// A convenience constructor to create a plain text notice.
    pub fn plain(body: impl Into<String>) -> Self {
        let body = body.into();
        Self {
            #[cfg(feature = "unstable-msc1767")]
            message: Some(MessageContent::plain(body.clone())),
            body,
            formatted: None,
        }
    }

    /// A convenience constructor to create an html notice.
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

    /// A convenience constructor to create a markdown notice.
    ///
    /// Returns an html notice if some markdown formatting was detected, otherwise returns a plain
    /// text notice.
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
impl From<MessageContent> for NoticeMessageEventContent {
    fn from(message: MessageContent) -> Self {
        let body = if let Some(body) = message.find_plain() { body } else { &message[0].body };
        let formatted = message.find_html().map(FormattedBody::html);

        Self { body: body.to_owned(), formatted, message: Some(message) }
    }
}
