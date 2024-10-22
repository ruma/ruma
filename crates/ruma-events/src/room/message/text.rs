use serde::{Deserialize, Serialize};

#[cfg(feature = "unstable-msc4095")]
use super::url_preview::UrlPreview;
use super::FormattedBody;

/// The payload for a text message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.text")]
pub struct TextMessageEventContent {
    /// The body of the message.
    pub body: String,

    /// Formatted form of the message `body`.
    #[serde(flatten)]
    pub formatted: Option<FormattedBody>,

    /// [MSC4095](https://github.com/matrix-org/matrix-spec-proposals/pull/4095)-style bundled url previews
    #[cfg(feature = "unstable-msc4095")]
    #[serde(
        rename(serialize = "com.beeper.linkpreviews"),
        skip_serializing_if = "Option::is_none",
        alias = "m.url_previews"
    )]
    pub url_previews: Option<Vec<UrlPreview>>,
}

impl TextMessageEventContent {
    /// A convenience constructor to create a plain text message.
    pub fn plain(body: impl Into<String>) -> Self {
        let body = body.into();
        Self {
            body,
            formatted: None,
            #[cfg(feature = "unstable-msc4095")]
            url_previews: None,
        }
    }

    /// A convenience constructor to create an HTML message.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        let body = body.into();
        Self {
            body,
            formatted: Some(FormattedBody::html(html_body)),
            #[cfg(feature = "unstable-msc4095")]
            url_previews: None,
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
