//! Types for extensible text message events ([MSC1767]).
//!
//! [MSC1767]: https://github.com/matrix-org/matrix-spec-proposals/pull/1767

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

mod content_serde;

use content_serde::MessageContentSerDeHelper;

use super::room::message::{FormattedBody, MessageFormat, Relation, TextMessageEventContent};

/// The payload for an extensible text message.
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.message", kind = MessageLike)]
pub struct MessageEventContent {
    /// The message's text content.
    #[serde(flatten)]
    pub message: MessageContent,

    /// Information about related messages for [rich replies].
    ///
    /// [rich replies]: https://spec.matrix.org/v1.2/client-server-api/#rich-replies
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Relation>,
}

impl MessageEventContent {
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

    /// Create a new `MessageEventContent` from the given `TextMessageEventContent` and optional
    /// relation.
    pub fn from_text_room_message(
        content: TextMessageEventContent,
        relates_to: Option<Relation>,
    ) -> Self {
        let TextMessageEventContent { body, formatted, message, .. } = content;
        if let Some(message) = message {
            Self { message, relates_to }
        } else {
            Self { message: MessageContent::from_room_message_content(body, formatted), relates_to }
        }
    }
}

/// Text message content.
#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "MessageContentSerDeHelper")]
pub struct MessageContent(pub(crate) Vec<Text>);

impl MessageContent {
    /// A convenience constructor to create a plain text message.
    pub fn plain(body: impl Into<String>) -> Self {
        Self(vec![Text::plain(body)])
    }

    /// A convenience constructor to create an HTML message.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self(vec![Text::html(html_body), Text::plain(body)])
    }

    /// A convenience constructor to create a Markdown message.
    ///
    /// Returns an HTML message if some Markdown formatting was detected, otherwise returns a plain
    /// text message.
    #[cfg(feature = "markdown")]
    pub fn markdown(body: impl AsRef<str> + Into<String>) -> Self {
        let mut message = Vec::with_capacity(2);
        if let Some(html_body) = Text::markdown(&body) {
            message.push(html_body);
        }
        message.push(Text::plain(body));
        Self(message)
    }

    /// Create a new `MessageContent` from the given body and optional formatted body.
    pub fn from_room_message_content(body: String, formatted: Option<FormattedBody>) -> Self {
        if let Some(FormattedBody { body: html_body, .. }) =
            formatted.filter(|formatted| formatted.format == MessageFormat::Html)
        {
            Self::html(body, html_body)
        } else {
            Self::plain(body)
        }
    }

    /// Get the plain text representation of this message.
    pub fn find_plain(&self) -> Option<&str> {
        self.variants()
            .iter()
            .find(|content| content.mimetype == "text/plain")
            .map(|content| content.body.as_ref())
    }

    /// Get the HTML representation of this message.
    pub fn find_html(&self) -> Option<&str> {
        self.variants()
            .iter()
            .find(|content| content.mimetype == "text/html")
            .map(|content| content.body.as_ref())
    }

    /// Get all the text representations of this message.
    pub fn variants(&self) -> &[Text] {
        &self.0
    }
}

impl From<Vec<Text>> for MessageContent {
    fn from(variants: Vec<Text>) -> Self {
        Self(variants)
    }
}

/// Text message content.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Text {
    /// The mime type of the `body`.
    #[serde(default = "Text::default_mimetype")]
    pub mimetype: String,

    /// The text content.
    pub body: String,

    /// The language of the text.
    ///
    /// This must be a valid language code according to [BCP 47](https://www.rfc-editor.org/rfc/bcp/bcp47.txt).
    #[cfg(feature = "unstable-msc3554")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

impl Text {
    /// Creates a new plain text message body.
    pub fn plain(body: impl Into<String>) -> Self {
        Self {
            mimetype: "text/plain".to_owned(),
            body: body.into(),
            #[cfg(feature = "unstable-msc3554")]
            lang: None,
        }
    }

    /// Creates a new HTML-formatted message body.
    pub fn html(body: impl Into<String>) -> Self {
        Self {
            mimetype: "text/html".to_owned(),
            body: body.into(),
            #[cfg(feature = "unstable-msc3554")]
            lang: None,
        }
    }

    /// Creates a new HTML-formatted message body by parsing the Markdown in `body`.
    ///
    /// Returns `None` if no Markdown formatting was found.
    #[cfg(feature = "markdown")]
    pub fn markdown(body: impl AsRef<str>) -> Option<Self> {
        let body = body.as_ref();
        let mut html_body = String::new();

        pulldown_cmark::html::push_html(&mut html_body, pulldown_cmark::Parser::new(body));

        (html_body != format!("<p>{}</p>\n", body)).then(|| Self::html(html_body))
    }

    fn default_mimetype() -> String {
        "text/plain".to_owned()
    }
}
