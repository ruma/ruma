//! Types for the [`m.room.message`] event.
//!
//! [`m.room.message`]: https://spec.matrix.org/latest/client-server-api/#mroommessage

use std::borrow::Cow;

use ruma_macros::EventContent;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::{
    events::relation::{InReplyTo, Replacement, Thread},
    serde::{JsonObject, StringEnum},
    OwnedEventId, PrivOwnedStr,
};

mod audio;
mod content_serde;
mod emote;
mod file;
mod image;
mod key_verification_request;
mod location;
mod notice;
pub(crate) mod relation_serde;
mod reply;
pub mod sanitize;
mod server_notice;
mod text;
mod video;

pub use audio::{AudioInfo, AudioMessageEventContent};
pub use emote::EmoteMessageEventContent;
pub use file::{FileInfo, FileMessageEventContent};
pub use image::ImageMessageEventContent;
pub use key_verification_request::KeyVerificationRequestEventContent;
pub use location::{LocationInfo, LocationMessageEventContent};
pub use notice::NoticeMessageEventContent;
pub use relation_serde::deserialize_relation;
#[cfg(feature = "unstable-sanitize")]
use sanitize::{
    remove_plain_reply_fallback, sanitize_html, HtmlSanitizerMode, RemoveReplyFallback,
};
pub use server_notice::{LimitType, ServerNoticeMessageEventContent, ServerNoticeType};
pub use text::TextMessageEventContent;
pub use video::{VideoInfo, VideoMessageEventContent};

/// The content of an `m.room.message` event.
///
/// This event is used when sending messages in a room.
///
/// Messages are not limited to be text.
#[derive(Clone, Debug, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.message", kind = MessageLike)]
pub struct RoomMessageEventContent {
    /// A key which identifies the type of message being sent.
    ///
    /// This also holds the specific content of each message.
    #[serde(flatten)]
    pub msgtype: MessageType,

    /// Information about related messages for [rich replies].
    ///
    /// [rich replies]: https://spec.matrix.org/latest/client-server-api/#rich-replies
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Relation<MessageType>>,
}

impl RoomMessageEventContent {
    /// Create a `RoomMessageEventContent` with the given `MessageType`.
    pub fn new(msgtype: MessageType) -> Self {
        Self { msgtype, relates_to: None }
    }

    /// A constructor to create a plain text message.
    pub fn text_plain(body: impl Into<String>) -> Self {
        Self::new(MessageType::text_plain(body))
    }

    /// A constructor to create an html message.
    pub fn text_html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self::new(MessageType::text_html(body, html_body))
    }

    /// A constructor to create a markdown message.
    #[cfg(feature = "markdown")]
    pub fn text_markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self::new(MessageType::text_markdown(body))
    }

    /// A constructor to create a plain text notice.
    pub fn notice_plain(body: impl Into<String>) -> Self {
        Self::new(MessageType::notice_plain(body))
    }

    /// A constructor to create an html notice.
    pub fn notice_html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self::new(MessageType::notice_html(body, html_body))
    }

    /// A constructor to create a markdown notice.
    #[cfg(feature = "markdown")]
    pub fn notice_markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self::new(MessageType::notice_markdown(body))
    }

    /// Turns `self` into a reply to the given message.
    ///
    /// Takes the `body` / `formatted_body` (if any) in `self` for the main text and prepends a
    /// quoted version of `original_message`. Also sets the `in_reply_to` field inside `relates_to`,
    /// and optionally the `rel_type` to `m.thread` if the `original_message is in a thread and
    /// thread forwarding is enabled.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/rich_reply.md"))]
    ///
    /// # Panics
    ///
    /// Panics if `self` has a `formatted_body` with a format other than HTML.
    #[track_caller]
    pub fn make_reply_to(
        mut self,
        original_message: &OriginalRoomMessageEvent,
        forward_thread: ForwardThread,
    ) -> Self {
        let empty_formatted_body = || FormattedBody::html(String::new());

        let (body, formatted) = {
            match &mut self.msgtype {
                MessageType::Emote(m) => {
                    (&mut m.body, Some(m.formatted.get_or_insert_with(empty_formatted_body)))
                }
                MessageType::Notice(m) => {
                    (&mut m.body, Some(m.formatted.get_or_insert_with(empty_formatted_body)))
                }
                MessageType::Text(m) => {
                    (&mut m.body, Some(m.formatted.get_or_insert_with(empty_formatted_body)))
                }
                MessageType::Audio(m) => (&mut m.body, None),
                MessageType::File(m) => (&mut m.body, None),
                MessageType::Image(m) => (&mut m.body, None),
                MessageType::Location(m) => (&mut m.body, None),
                MessageType::ServerNotice(m) => (&mut m.body, None),
                MessageType::Video(m) => (&mut m.body, None),
                MessageType::VerificationRequest(m) => (&mut m.body, None),
                MessageType::_Custom(m) => (&mut m.body, None),
            }
        };

        if let Some(f) = formatted {
            assert_eq!(
                f.format,
                MessageFormat::Html,
                "make_reply_to can't handle non-HTML formatted messages"
            );

            let formatted_body = &mut f.body;

            (*body, *formatted_body) = reply::plain_and_formatted_reply_body(
                body.as_str(),
                (!formatted_body.is_empty()).then_some(formatted_body.as_str()),
                original_message,
            );
        }

        let relates_to = if let Some(Relation::Thread(Thread { event_id, .. })) = original_message
            .content
            .relates_to
            .as_ref()
            .filter(|_| forward_thread == ForwardThread::Yes)
        {
            Relation::Thread(Thread::plain(event_id.clone(), original_message.event_id.clone()))
        } else {
            Relation::Reply {
                in_reply_to: InReplyTo { event_id: original_message.event_id.clone() },
            }
        };
        self.relates_to = Some(relates_to);

        self
    }

    /// Turns `self` into a new message for a thread, that is optionally a reply.
    ///
    /// Looks for a [`Relation::Thread`] in `previous_message`. If it exists, this message will be
    /// in the same thread. If it doesn't, a new thread with `previous_message` as the root is
    /// created.
    ///
    /// If this is a reply within the thread, takes the `body` / `formatted_body` (if any) in `self`
    /// for the main text and prepends a quoted version of `previous_message`. Also sets the
    /// `in_reply_to` field inside `relates_to`.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/rich_reply.md"))]
    ///
    /// # Panics
    ///
    /// Panics if this is a reply within the thread and `self` has a `formatted_body` with a format
    /// other than HTML.
    pub fn make_for_thread(
        mut self,
        previous_message: &OriginalRoomMessageEvent,
        is_reply: ReplyWithinThread,
    ) -> Self {
        if is_reply == ReplyWithinThread::Yes {
            self = self.make_reply_to(previous_message, ForwardThread::No);
        }

        let thread_root = if let Some(Relation::Thread(Thread { event_id, .. })) =
            &previous_message.content.relates_to
        {
            event_id.clone()
        } else {
            previous_message.event_id.clone()
        };

        self.relates_to = Some(Relation::Thread(Thread {
            event_id: thread_root,
            in_reply_to: InReplyTo { event_id: previous_message.event_id.clone() },
            is_falling_back: is_reply == ReplyWithinThread::No,
        }));

        self
    }

    /// Turns `self` into a [replacement] (or edit) for the message with the given event ID.
    ///
    /// This takes the content and sets it in `m.new_content`, and modifies the `content` to include
    /// a fallback.
    ///
    /// If the message that is replaced is a reply to another message, the latter should also be
    /// provided to be able to generate a rich reply fallback that takes the `body` /
    /// `formatted_body` (if any) in `self` for the main text and prepends a quoted version of
    /// `original_message`.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/rich_reply.md"))]
    ///
    /// # Panics
    ///
    /// Panics if `self` has a `formatted_body` with a format other than HTML.
    ///
    /// [replacement]: https://spec.matrix.org/latest/client-server-api/#event-replacements
    #[track_caller]
    pub fn make_replacement(
        mut self,
        original_message_id: OwnedEventId,
        replied_to_message: Option<&OriginalRoomMessageEvent>,
    ) -> Self {
        // Prepare relates_to with the untouched msgtype.
        let relates_to = Relation::Replacement(Replacement {
            event_id: original_message_id,
            new_content: self.msgtype.clone(),
        });

        let empty_formatted_body = || FormattedBody::html(String::new());

        let (body, formatted) = {
            match &mut self.msgtype {
                MessageType::Emote(m) => {
                    (&mut m.body, Some(m.formatted.get_or_insert_with(empty_formatted_body)))
                }
                MessageType::Notice(m) => {
                    (&mut m.body, Some(m.formatted.get_or_insert_with(empty_formatted_body)))
                }
                MessageType::Text(m) => {
                    (&mut m.body, Some(m.formatted.get_or_insert_with(empty_formatted_body)))
                }
                MessageType::Audio(m) => (&mut m.body, None),
                MessageType::File(m) => (&mut m.body, None),
                MessageType::Image(m) => (&mut m.body, None),
                MessageType::Location(m) => (&mut m.body, None),
                MessageType::ServerNotice(m) => (&mut m.body, None),
                MessageType::Video(m) => (&mut m.body, None),
                MessageType::VerificationRequest(m) => (&mut m.body, None),
                MessageType::_Custom(m) => (&mut m.body, None),
            }
        };

        // Add replacement fallback.
        *body = format!("* {body}");

        if let Some(f) = formatted {
            assert_eq!(
                f.format,
                MessageFormat::Html,
                "make_replacement can't handle non-HTML formatted messages"
            );

            f.body = format!("* {}", f.body);
        }

        // Add reply fallback if needed.
        if let Some(original_message) = replied_to_message {
            self = self.make_reply_to(original_message, ForwardThread::No);
        }

        self.relates_to = Some(relates_to);

        self
    }

    /// Returns a reference to the `msgtype` string.
    ///
    /// If you want to access the message type-specific data rather than the message type itself,
    /// use the `msgtype` *field*, not this method.
    pub fn msgtype(&self) -> &str {
        self.msgtype.msgtype()
    }

    /// Return a reference to the message body.
    pub fn body(&self) -> &str {
        self.msgtype.body()
    }

    /// Sanitize this message.
    ///
    /// If this message contains HTML, this removes the [tags and attributes] that are not listed in
    /// the Matrix specification.
    ///
    /// It can also optionally remove the [rich reply fallback] from the plain text and HTML
    /// message.
    ///
    /// This method is only effective on text, notice and emote messages.
    ///
    /// [tags and attributes]: https://spec.matrix.org/latest/client-server-api/#mroommessage-msgtypes
    /// [rich reply fallback]: https://spec.matrix.org/latest/client-server-api/#fallbacks-for-rich-replies
    #[cfg(feature = "unstable-sanitize")]
    pub fn sanitize(
        &mut self,
        mode: HtmlSanitizerMode,
        remove_reply_fallback: RemoveReplyFallback,
    ) {
        if let MessageType::Emote(EmoteMessageEventContent { body, formatted, .. })
        | MessageType::Notice(NoticeMessageEventContent { body, formatted, .. })
        | MessageType::Text(TextMessageEventContent { body, formatted, .. }) = &mut self.msgtype
        {
            if let Some(formatted) = formatted {
                formatted.sanitize_html(mode, remove_reply_fallback);
            }
            if remove_reply_fallback == RemoveReplyFallback::Yes
                && matches!(self.relates_to, Some(Relation::Reply { .. }))
            {
                *body = remove_plain_reply_fallback(body).to_owned();
            }
        }
    }
}

/// Whether or not to forward a [`Relation::Thread`] when sending a reply.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
pub enum ForwardThread {
    /// The thread relation in the original message is forwarded if it exists.
    ///
    /// This should be set if your client doesn't render threads (see the [info
    /// box for clients which are acutely aware of threads]).
    ///
    /// [info box for clients which are acutely aware of threads]: https://spec.matrix.org/latest/client-server-api/#fallback-for-unthreaded-clients
    Yes,

    /// Create a reply in the main conversation even if the original message is in a thread.
    ///
    /// This should be used if you client supports threads and you explicitly want that behavior.
    No,
}

/// Whether or not the message is a reply inside a thread.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
pub enum ReplyWithinThread {
    /// This is a reply.
    ///
    /// Create a [reply within the thread].
    ///
    /// [reply within the thread]: https://spec.matrix.org/latest/client-server-api/#replies-within-threads
    Yes,

    /// This is not a reply.
    ///
    /// Create a regular message in the thread, with a [fallback for unthreaded clients].
    ///
    /// [fallback for unthreaded clients]: https://spec.matrix.org/latest/client-server-api/#fallback-for-unthreaded-clients
    No,
}

/// The content that is specific to each message type variant.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum MessageType {
    /// An audio message.
    Audio(AudioMessageEventContent),

    /// An emote message.
    Emote(EmoteMessageEventContent),

    /// A file message.
    File(FileMessageEventContent),

    /// An image message.
    Image(ImageMessageEventContent),

    /// A location message.
    Location(LocationMessageEventContent),

    /// A notice message.
    Notice(NoticeMessageEventContent),

    /// A server notice message.
    ServerNotice(ServerNoticeMessageEventContent),

    /// A text message.
    Text(TextMessageEventContent),

    /// A video message.
    Video(VideoMessageEventContent),

    /// A request to initiate a key verification.
    VerificationRequest(KeyVerificationRequestEventContent),

    /// A custom message.
    #[doc(hidden)]
    _Custom(CustomEventContent),
}

impl MessageType {
    /// Creates a new `MessageType`.
    ///
    /// The `msgtype` and `body` are required fields as defined by [the `m.room.message` spec](https://spec.matrix.org/latest/client-server-api/#mroommessage).
    /// Additionally it's possible to add arbitrary key/value pairs to the event content for custom
    /// events through the `data` map.
    ///
    /// Prefer to use the public variants of `MessageType` where possible; this constructor is meant
    /// be used for unsupported message types only and does not allow setting arbitrary data for
    /// supported ones.
    ///
    /// # Errors
    ///
    /// Returns an error if the `msgtype` is known and serialization of `data` to the corresponding
    /// `MessageType` variant fails.
    pub fn new(msgtype: &str, body: String, data: JsonObject) -> serde_json::Result<Self> {
        fn deserialize_variant<T: DeserializeOwned>(
            body: String,
            mut obj: JsonObject,
        ) -> serde_json::Result<T> {
            obj.insert("body".into(), body.into());
            serde_json::from_value(JsonValue::Object(obj))
        }

        Ok(match msgtype {
            "m.audio" => Self::Audio(deserialize_variant(body, data)?),
            "m.emote" => Self::Emote(deserialize_variant(body, data)?),
            "m.file" => Self::File(deserialize_variant(body, data)?),
            "m.image" => Self::Image(deserialize_variant(body, data)?),
            "m.location" => Self::Location(deserialize_variant(body, data)?),
            "m.notice" => Self::Notice(deserialize_variant(body, data)?),
            "m.server_notice" => Self::ServerNotice(deserialize_variant(body, data)?),
            "m.text" => Self::Text(deserialize_variant(body, data)?),
            "m.video" => Self::Video(deserialize_variant(body, data)?),
            "m.key.verification.request" => {
                Self::VerificationRequest(deserialize_variant(body, data)?)
            }
            _ => Self::_Custom(CustomEventContent { msgtype: msgtype.to_owned(), body, data }),
        })
    }

    /// A constructor to create a plain text message.
    pub fn text_plain(body: impl Into<String>) -> Self {
        Self::Text(TextMessageEventContent::plain(body))
    }

    /// A constructor to create an html message.
    pub fn text_html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self::Text(TextMessageEventContent::html(body, html_body))
    }

    /// A constructor to create a markdown message.
    #[cfg(feature = "markdown")]
    pub fn text_markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self::Text(TextMessageEventContent::markdown(body))
    }

    /// A constructor to create a plain text notice.
    pub fn notice_plain(body: impl Into<String>) -> Self {
        Self::Notice(NoticeMessageEventContent::plain(body))
    }

    /// A constructor to create an html notice.
    pub fn notice_html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self::Notice(NoticeMessageEventContent::html(body, html_body))
    }

    /// A constructor to create a markdown notice.
    #[cfg(feature = "markdown")]
    pub fn notice_markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self::Notice(NoticeMessageEventContent::markdown(body))
    }

    /// Returns a reference to the `msgtype` string.
    pub fn msgtype(&self) -> &str {
        match self {
            Self::Audio(_) => "m.audio",
            Self::Emote(_) => "m.emote",
            Self::File(_) => "m.file",
            Self::Image(_) => "m.image",
            Self::Location(_) => "m.location",
            Self::Notice(_) => "m.notice",
            Self::ServerNotice(_) => "m.server_notice",
            Self::Text(_) => "m.text",
            Self::Video(_) => "m.video",
            Self::VerificationRequest(_) => "m.key.verification.request",
            Self::_Custom(c) => &c.msgtype,
        }
    }

    /// Return a reference to the message body.
    pub fn body(&self) -> &str {
        match self {
            MessageType::Audio(m) => &m.body,
            MessageType::Emote(m) => &m.body,
            MessageType::File(m) => &m.body,
            MessageType::Image(m) => &m.body,
            MessageType::Location(m) => &m.body,
            MessageType::Notice(m) => &m.body,
            MessageType::ServerNotice(m) => &m.body,
            MessageType::Text(m) => &m.body,
            MessageType::Video(m) => &m.body,
            MessageType::VerificationRequest(m) => &m.body,
            MessageType::_Custom(m) => &m.body,
        }
    }

    /// Returns the associated data.
    ///
    /// The returned JSON object won't contain the `msgtype` and `body` fields, use
    /// [`.msgtype()`][Self::msgtype] / [`.body()`](Self::body) to access those.
    ///
    /// Prefer to use the public variants of `MessageType` where possible; this method is meant to
    /// be used for custom message types only.
    pub fn data(&self) -> Cow<'_, JsonObject> {
        fn serialize<T: Serialize>(obj: &T) -> JsonObject {
            match serde_json::to_value(obj).expect("message type serialization to succeed") {
                JsonValue::Object(mut obj) => {
                    obj.remove("body");
                    obj
                }
                _ => panic!("all message types must serialize to objects"),
            }
        }

        match self {
            Self::Audio(d) => Cow::Owned(serialize(d)),
            Self::Emote(d) => Cow::Owned(serialize(d)),
            Self::File(d) => Cow::Owned(serialize(d)),
            Self::Image(d) => Cow::Owned(serialize(d)),
            Self::Location(d) => Cow::Owned(serialize(d)),
            Self::Notice(d) => Cow::Owned(serialize(d)),
            Self::ServerNotice(d) => Cow::Owned(serialize(d)),
            Self::Text(d) => Cow::Owned(serialize(d)),
            Self::Video(d) => Cow::Owned(serialize(d)),
            Self::VerificationRequest(d) => Cow::Owned(serialize(d)),
            Self::_Custom(c) => Cow::Borrowed(&c.data),
        }
    }
}

impl From<MessageType> for RoomMessageEventContent {
    fn from(msgtype: MessageType) -> Self {
        Self::new(msgtype)
    }
}

impl From<RoomMessageEventContent> for MessageType {
    fn from(content: RoomMessageEventContent) -> Self {
        content.msgtype
    }
}

/// Message event relationship.
#[derive(Clone, Debug)]
#[allow(clippy::manual_non_exhaustive)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum Relation<C> {
    /// An `m.in_reply_to` relation indicating that the event is a reply to another event.
    Reply {
        /// Information about another message being replied to.
        in_reply_to: InReplyTo,
    },

    /// An event that replaces another event.
    Replacement(Replacement<C>),

    /// An event that belongs to a thread.
    Thread(Thread),

    #[doc(hidden)]
    _Custom,
}

/// The format for the formatted representation of a message body.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, StringEnum)]
#[non_exhaustive]
pub enum MessageFormat {
    /// HTML.
    #[ruma_enum(rename = "org.matrix.custom.html")]
    Html,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// Common message event content fields for message types that have separate plain-text and
/// formatted representations.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::exhaustive_structs)]
pub struct FormattedBody {
    /// The format used in the `formatted_body`.
    pub format: MessageFormat,

    /// The formatted version of the `body`.
    #[serde(rename = "formatted_body")]
    pub body: String,
}

impl FormattedBody {
    /// Creates a new HTML-formatted message body.
    pub fn html(body: impl Into<String>) -> Self {
        Self { format: MessageFormat::Html, body: body.into() }
    }

    /// Creates a new HTML-formatted message body by parsing the Markdown in `body`.
    ///
    /// Returns `None` if no Markdown formatting was found.
    #[cfg(feature = "markdown")]
    pub fn markdown(body: impl AsRef<str>) -> Option<Self> {
        parse_markdown(body.as_ref()).map(Self::html)
    }

    /// Sanitize this `FormattedBody` if its format is `MessageFormat::Html`.
    ///
    /// This removes any [tags and attributes] that are not listed in the Matrix specification.
    ///
    /// It can also optionally remove the [rich reply fallback].
    ///
    /// Returns the sanitized HTML if the format is `MessageFormat::Html`.
    ///
    /// [tags and attributes]: https://spec.matrix.org/latest/client-server-api/#mroommessage-msgtypes
    /// [rich reply fallback]: https://spec.matrix.org/latest/client-server-api/#fallbacks-for-rich-replies
    #[cfg(feature = "unstable-sanitize")]
    pub fn sanitize_html(
        &mut self,
        mode: HtmlSanitizerMode,
        remove_reply_fallback: RemoveReplyFallback,
    ) {
        if self.format == MessageFormat::Html {
            self.body = sanitize_html(&self.body, mode, remove_reply_fallback);
        }
    }
}

/// The payload for a custom message event.
#[doc(hidden)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CustomEventContent {
    /// A custom msgtype.
    msgtype: String,

    /// The message body.
    body: String,

    /// Remaining event content.
    #[serde(flatten)]
    data: JsonObject,
}

#[cfg(feature = "markdown")]
pub(crate) fn parse_markdown(text: &str) -> Option<String> {
    use pulldown_cmark::{Event, Options, Parser, Tag};

    const OPTIONS: Options = Options::ENABLE_TABLES.union(Options::ENABLE_STRIKETHROUGH);

    let mut found_first_paragraph = false;

    let has_markdown = Parser::new_ext(text, OPTIONS).any(|ref event| {
        let is_text = matches!(event, Event::Text(_));
        let is_break = matches!(event, Event::SoftBreak | Event::HardBreak);
        let is_first_paragraph_start = if matches!(event,
            Event::Start(tag)
                if matches!(tag, Tag::Paragraph)
        ) {
            if found_first_paragraph {
                false
            } else {
                found_first_paragraph = true;
                true
            }
        } else {
            false
        };
        let is_paragraph_end = matches!(event,
                Event::End(tag)
                if matches!(tag, Tag::Paragraph)
        );

        !is_text && !is_break && !is_first_paragraph_start && !is_paragraph_end
    });

    if !has_markdown {
        return None;
    }

    let mut html_body = String::new();
    pulldown_cmark::html::push_html(&mut html_body, Parser::new_ext(text, OPTIONS));

    Some(html_body)
}
