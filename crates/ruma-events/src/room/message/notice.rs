use serde::{Deserialize, Serialize};

use super::FormattedBody;

/// The payload for a notice message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.notice")]
pub struct NoticeMessageEventContent {
    /// The notice text.
    pub body: String,

    /// Formatted form of the message `body`.
    #[serde(flatten)]
    pub formatted: Option<FormattedBody>,
}

impl NoticeMessageEventContent {
    /// A convenience constructor to create a plain text notice.
    pub fn plain(body: impl Into<String>) -> Self {
        let body = body.into();
        Self { body, formatted: None }
    }

    /// A convenience constructor to create an html notice.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        let body = body.into();
        Self { body, formatted: Some(FormattedBody::html(html_body)) }
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
