//! Types for extensible text message events ([MSC1767]).
//!
//! # Extensible events
//!
//! [MSC1767] defines a new structure for events that is made of two parts: a type and zero or more
//! reusable content blocks.
//!
//! This allows to construct new event types from a list of known content blocks that allows in turn
//! clients to be able to render unknown event types by using the known content blocks as a
//! fallback. When a new type is defined, all the content blocks it can or must contain are defined
//! too.
//!
//! There are also some content blocks called "mixins" that can apply to any event when they are
//! defined.
//!
//! # MSCs
//!
//! This is a list of MSCs defining the extensible events and deprecating the corresponding legacy
//! types. Note that "primary type" means the `type` field at the root of the event and "message
//! type" means the `msgtype` field in the content of the `m.room.message` primary type.
//!
//! - [MSC1767]: Text messages, where the `m.message` primary type replaces the `m.text` message
//!   type.
//! - [MSC3954]: Emotes, where the `m.emote` primary type replaces the `m.emote` message type.
//! - [MSC3955]: Automated events, where the `m.automated` mixin replaces the `m.notice` message
//!   type.
//! - [MSC3956]: Encrypted events, where the `m.encrypted` primary type replaces the
//!   `m.room.encrypted` primary type.
//! - [MSC3551]: Files, where the `m.file` primary type replaces the `m.file` message type.
//! - [MSC3552]: Images and Stickers, where the `m.image` primary type replaces the `m.image`
//!   message type and the `m.sticker` primary type.
//! - [MSC3553]: Videos, where the `m.video` primary type replaces the `m.video` message type.
//! - [MSC3927]: Audio, where the `m.audio` primary type replaces the `m.audio` message type.
//! - [MSC3488]: Location, where the `m.location` primary type replaces the `m.location` message
//!   type.
//!
//! There are also the following MSCs that introduce new features with extensible events:
//!
//! - [MSC3245]: Voice Messages.
//! - [MSC3246]: Audio Waveform.
//! - [MSC3381]: Polls.
//!
//! # How to use them in Matrix
//!
//! The extensible events types are meant to be used separately than the legacy types. As such,
//! their use is reserved for room versions that support it.
//!
//! Currently no stable room version supports extensible events so they can only be sent with
//! unstable room versions that support them.
//!
//! An exception is made for some new extensible events types that don't have a legacy type. They
//! can be used with stable room versions without support for extensible types, but they might be
//! ignored by clients that have no support for extensible events. The types that support this must
//! advertise it in their MSC.
//!
//! Note that if a room version supports extensible events, it doesn't support the legacy types
//! anymore and those should be ignored. There is not yet a definition of the deprecated legacy
//! types in extensible events rooms.
//!
//! # How to use them in Ruma
//!
//! First, you can enable the `unstable-extensible-events` feature from the `ruma` crate, that
//! will enable all the MSCs for the extensible events that correspond to the legacy types. It
//! is also possible to enable only the MSCs you want with the `unstable-mscXXXX` features (where
//! `XXXX` is the number of the MSC). When enabling an MSC, all MSC dependencies are enabled at the
//! same time to avoid issues.
//!
//! Currently the extensible events use the unstable prefixes as defined in the corresponding MSCs.
//!
//! [MSC1767]: https://github.com/matrix-org/matrix-spec-proposals/pull/1767
//! [MSC3954]: https://github.com/matrix-org/matrix-spec-proposals/pull/3954
//! [MSC3955]: https://github.com/matrix-org/matrix-spec-proposals/pull/3955
//! [MSC3956]: https://github.com/matrix-org/matrix-spec-proposals/pull/3956
//! [MSC3551]: https://github.com/matrix-org/matrix-spec-proposals/pull/3551
//! [MSC3552]: https://github.com/matrix-org/matrix-spec-proposals/pull/3552
//! [MSC3553]: https://github.com/matrix-org/matrix-spec-proposals/pull/3553
//! [MSC3927]: https://github.com/matrix-org/matrix-spec-proposals/pull/3927
//! [MSC3488]: https://github.com/matrix-org/matrix-spec-proposals/pull/3488
//! [MSC3245]: https://github.com/matrix-org/matrix-spec-proposals/pull/3245
//! [MSC3246]: https://github.com/matrix-org/matrix-spec-proposals/pull/3246
//! [MSC3381]: https://github.com/matrix-org/matrix-spec-proposals/pull/3381
use std::ops::Deref;

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::room::message::Relation;
#[cfg(feature = "unstable-msc4095")]
use super::room::message::UrlPreview;

pub(super) mod historical_serde;

/// The payload for an extensible text message.
///
/// This is the new primary type introduced in [MSC1767] and should only be sent in rooms with a
/// version that supports it. See the documentation of the [`message`] module for more information.
///
/// To construct a `MessageEventContent` with a custom [`TextContentBlock`], convert it with
/// `MessageEventContent::from()` / `.into()`.
///
/// [MSC1767]: https://github.com/matrix-org/matrix-spec-proposals/pull/1767
/// [`message`]: super::message
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc1767.message", kind = MessageLike, without_relation)]
pub struct MessageEventContent {
    /// The message's text content.
    #[serde(rename = "org.matrix.msc1767.text", alias = "m.text")]
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
    pub relates_to: Option<Relation<MessageEventContentWithoutRelation>>,

    /// [MSC4095](https://github.com/matrix-org/matrix-spec-proposals/pull/4095)-style bundled url previews
    #[cfg(feature = "unstable-msc4095")]
    #[serde(
        rename(serialize = "com.beeper.linkpreviews"),
        skip_serializing_if = "Option::is_none",
        alias = "m.url_previews"
    )]
    pub url_previews: Option<Vec<UrlPreview>>,
}

impl MessageEventContent {
    /// A convenience constructor to create a plain text message.
    pub fn plain(body: impl Into<String>) -> Self {
        Self {
            text: TextContentBlock::plain(body),
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
            relates_to: None,
            #[cfg(feature = "unstable-msc4095")]
            url_previews: None,
        }
    }

    /// A convenience constructor to create an HTML message.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self {
            text: TextContentBlock::html(body, html_body),
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
            relates_to: None,
            #[cfg(feature = "unstable-msc4095")]
            url_previews: None,
        }
    }

    /// A convenience constructor to create a message from Markdown.
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
            #[cfg(feature = "unstable-msc4095")]
            url_previews: None,
        }
    }
}

impl From<TextContentBlock> for MessageEventContent {
    fn from(text: TextContentBlock) -> Self {
        Self {
            text,
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
            relates_to: None,
            #[cfg(feature = "unstable-msc4095")]
            url_previews: None,
        }
    }
}

/// A block for text content with optional markup.
///
/// This is an array of [`TextRepresentation`].
///
/// To construct a `TextContentBlock` with custom MIME types, construct a `Vec<TextRepresentation>`
/// first and use its `::from()` / `.into()` implementation.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TextContentBlock(Vec<TextRepresentation>);

impl TextContentBlock {
    /// A convenience constructor to create a plain text message.
    pub fn plain(body: impl Into<String>) -> Self {
        Self(vec![TextRepresentation::plain(body)])
    }

    /// A convenience constructor to create an HTML message with a plain text fallback.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self(vec![TextRepresentation::html(html_body), TextRepresentation::plain(body)])
    }

    /// A convenience constructor to create a message from Markdown.
    ///
    /// The content includes an HTML message if some Markdown formatting was detected, otherwise
    /// only a plain text message is included.
    #[cfg(feature = "markdown")]
    pub fn markdown(body: impl AsRef<str> + Into<String>) -> Self {
        let mut message = Vec::with_capacity(2);
        if let Some(html_body) = TextRepresentation::markdown(&body) {
            message.push(html_body);
        }
        message.push(TextRepresentation::plain(body));
        Self(message)
    }

    /// Whether this content block is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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

impl From<Vec<TextRepresentation>> for TextContentBlock {
    fn from(representations: Vec<TextRepresentation>) -> Self {
        Self(representations)
    }
}

impl FromIterator<TextRepresentation> for TextContentBlock {
    fn from_iter<T: IntoIterator<Item = TextRepresentation>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Deref for TextContentBlock {
    type Target = [TextRepresentation];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Text content with optional markup.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct TextRepresentation {
    /// The MIME type of the `body`.
    ///
    /// This must follow the format defined in [RFC6838].
    ///
    /// [RFC6838]: https://datatracker.ietf.org/doc/html/rfc6838
    #[serde(
        default = "TextRepresentation::default_mimetype",
        skip_serializing_if = "TextRepresentation::is_default_mimetype"
    )]
    pub mimetype: String,

    /// The text content.
    pub body: String,

    /// The language of the text ([MSC3554]).
    ///
    /// This must be a valid language code according to [BCP 47](https://www.rfc-editor.org/rfc/bcp/bcp47.txt).
    ///
    /// This is optional and defaults to `en`.
    ///
    /// [MSC3554]: https://github.com/matrix-org/matrix-spec-proposals/pull/3554
    #[cfg(feature = "unstable-msc3554")]
    #[serde(
        rename = "org.matrix.msc3554.lang",
        default = "TextRepresentation::default_lang",
        skip_serializing_if = "TextRepresentation::is_default_lang"
    )]
    pub lang: String,
}

impl TextRepresentation {
    /// Creates a new `TextRepresentation` with the given MIME type and body.
    pub fn new(mimetype: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            mimetype: mimetype.into(),
            body: body.into(),
            #[cfg(feature = "unstable-msc3554")]
            lang: Self::default_lang(),
        }
    }

    /// Creates a new plain text message body.
    pub fn plain(body: impl Into<String>) -> Self {
        Self::new("text/plain", body)
    }

    /// Creates a new HTML-formatted message body.
    pub fn html(body: impl Into<String>) -> Self {
        Self::new("text/html", body)
    }

    /// Creates a new HTML-formatted message body by parsing the Markdown in `body`.
    ///
    /// Returns `None` if no Markdown formatting was found.
    #[cfg(feature = "markdown")]
    pub fn markdown(body: impl AsRef<str>) -> Option<Self> {
        use super::room::message::parse_markdown;

        parse_markdown(body.as_ref()).map(Self::html)
    }

    fn default_mimetype() -> String {
        "text/plain".to_owned()
    }

    fn is_default_mimetype(mime: &str) -> bool {
        mime == "text/plain"
    }

    #[cfg(feature = "unstable-msc3554")]
    fn default_lang() -> String {
        "en".to_owned()
    }

    #[cfg(feature = "unstable-msc3554")]
    fn is_default_lang(lang: &str) -> bool {
        lang == "en"
    }
}
