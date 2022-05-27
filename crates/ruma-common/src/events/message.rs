//! Types for extensible text message events ([MSC1767]).
//!
//! # Extensible events
//!
//! MSCs [1767] (Text, Emote and Notice), [3551] (Files), [3552] (Images and Stickers), [3553]
//! (Videos), [3246] (Audio), and [3488] (Location) introduce new primary types called extensible
//! events. These types are meant to replace the `m.room.message` primary type and its `msgtype`s.
//! Other MSCs introduce new types with an `m.room.message` fallback, like [MSC3245] (Voice
//! Messages), and types that only have an extensible events format, like [MSC3381] (Polls).
//!
//! # Transition Period
//!
//! MSC1767 defines a transition period that will start after the extensible events are released in
//! a Matrix version. It should last approximately one year, but the end of that period will be
//! formalized in a new Matrix version.
//!
//! The new primary types should not be sent over the Matrix network before the end of the
//! transition period. Instead, transitional `m.room.message` events should be sent. These
//! transitional events include the content of the now legacy `m.room.message` event and the content
//! of the new extensible event types in a single event.
//!
//! # How to use them
//!
//! First, you can enable the `unstable-extensible-events` feature from the `ruma` crate, that
//! will enable all the MSCs for the extensible events that correspond to the legacy `msgtype`s
//! (1767, 3246, 3488, 3551, 3552, 3553). It is also possible to enable only the MSCs you want with
//! the `unstable-mscXXXX` features (where `XXXX` is the number of the MSC).
//!
//! The recommended way to send transitional extensible events while they are unstable and during
//! the transition period is to build one of the new primary types and then to convert it to a
//! [`RoomMessageEventContent`] by using `.into()` or `RoomMessageEventContent::from()`. The
//! provided constructors will copy the relevant data in the legacy fields.
//!
//! For incoming events, a `RoomMessageEventContent` can be converted to an extensible event with
//! the relevant `from_*_room_message` method on the primary type. This conversion will work even
//! with legacy `m.room.message` events that don't have extensible events content.
//!
//! It is also possible to enable extensible events support and continue using
//! `RoomMessageEventContent`'s constructors. The data will be duplicated in both the legacy and
//! extensible events fields.
//!
//! [MSC1767]: https://github.com/matrix-org/matrix-spec-proposals/pull/1767
//! [1767]: https://github.com/matrix-org/matrix-spec-proposals/pull/1767
//! [3551]: https://github.com/matrix-org/matrix-spec-proposals/pull/3551
//! [3552]: https://github.com/matrix-org/matrix-spec-proposals/pull/3552
//! [3553]: https://github.com/matrix-org/matrix-spec-proposals/pull/3553
//! [3246]: https://github.com/matrix-org/matrix-spec-proposals/pull/3246
//! [3488]: https://github.com/matrix-org/matrix-spec-proposals/pull/3488
//! [MSC3245]: https://github.com/matrix-org/matrix-spec-proposals/pull/3245
//! [MSC3381]: https://github.com/matrix-org/matrix-spec-proposals/pull/3381
//! [`RoomMessageEventContent`]: super::room::message::RoomMessageEventContent
use std::ops::Deref;

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub(crate) mod content_serde;

use content_serde::MessageContentSerDeHelper;

use super::room::message::{FormattedBody, MessageFormat, Relation, TextMessageEventContent};

/// The payload for an extensible text message.
///
/// This is the new primary type introduced in [MSC1767] and should not be sent before the end of
/// the transition period. See the documentation of the [`message`] module for more information.
///
/// `MessageEventContent` can be converted to a [`RoomMessageEventContent`] with a
/// [`MessageType::Text`]. You can convert it back with
/// [`MessageEventContent::from_text_room_message()`].
///
/// [MSC1767]: https://github.com/matrix-org/matrix-spec-proposals/pull/1767
/// [`message`]: super::message
/// [`RoomMessageEventContent`]: super::room::message::RoomMessageEventContent
/// [`MessageType::Text`]: super::room::message::MessageType::Text
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.message", kind = MessageLike)]
pub struct MessageEventContent {
    /// The message's text content.
    #[serde(flatten)]
    pub message: MessageContent,

    /// Information about related messages.
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
///
/// A `MessageContent` must contain at least one message to be used as a fallback text
/// representation.
#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "MessageContentSerDeHelper")]
pub struct MessageContent(pub(crate) Vec<Text>);

impl MessageContent {
    /// Create a `MessageContent` from an array of messages.
    ///
    /// Returns `None` if the array is empty.
    pub fn new(messages: Vec<Text>) -> Option<Self> {
        if messages.is_empty() {
            None
        } else {
            Some(Self(messages))
        }
    }

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
        self.iter()
            .find(|content| content.mimetype == "text/plain")
            .map(|content| content.body.as_ref())
    }

    /// Get the HTML representation of this message.
    pub fn find_html(&self) -> Option<&str> {
        self.iter()
            .find(|content| content.mimetype == "text/html")
            .map(|content| content.body.as_ref())
    }
}

/// The error type returned when trying to construct an empty `MessageContent`.
#[derive(Debug, Error)]
#[non_exhaustive]
#[error("MessageContent cannot be empty")]
pub struct EmptyMessageContentError;

impl TryFrom<Vec<Text>> for MessageContent {
    type Error = EmptyMessageContentError;

    fn try_from(messages: Vec<Text>) -> Result<Self, Self::Error> {
        Self::new(messages).ok_or(EmptyMessageContentError)
    }
}

impl Deref for MessageContent {
    type Target = [Text];

    fn deref(&self) -> &Self::Target {
        &self.0
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

    /// The language of the text ([MSC3554]).
    ///
    /// This must be a valid language code according to [BCP 47](https://www.rfc-editor.org/rfc/bcp/bcp47.txt).
    ///
    /// [MSC3554]: https://github.com/matrix-org/matrix-spec-proposals/pull/3554
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

/// The error type returned when a conversion to an extensible event type fails.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum TryFromExtensibleError {
    /// A field is missing.
    #[error("missing field `{0}`")]
    MissingField(String),
}
