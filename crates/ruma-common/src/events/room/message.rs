//! Types for the [`m.room.message`] event.
//!
//! [`m.room.message`]: https://spec.matrix.org/v1.2/client-server-api/#mroommessage

use std::borrow::Cow;

use ruma_macros::EventContent;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::{
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
mod relation_serde;
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
    /// [rich replies]: https://spec.matrix.org/v1.2/client-server-api/#rich-replies
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Relation>,
}

impl RoomMessageEventContent {
    /// Create a `RoomMessageEventContent` with the given `MessageType`.
    pub fn new(msgtype: MessageType) -> Self {
        Self { msgtype, relates_to: None }
    }

    /// A constructor to create a plain text message.
    pub fn text_plain(body: impl Into<String>) -> Self {
        Self::new(MessageType::Text(TextMessageEventContent::plain(body)))
    }

    /// A constructor to create an html message.
    pub fn text_html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self::new(MessageType::Text(TextMessageEventContent::html(body, html_body)))
    }

    /// A constructor to create a markdown message.
    #[cfg(feature = "markdown")]
    pub fn text_markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self::new(MessageType::Text(TextMessageEventContent::markdown(body)))
    }

    /// A constructor to create a plain text notice.
    pub fn notice_plain(body: impl Into<String>) -> Self {
        Self::new(MessageType::Notice(NoticeMessageEventContent::plain(body)))
    }

    /// A constructor to create an html notice.
    pub fn notice_html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self::new(MessageType::Notice(NoticeMessageEventContent::html(body, html_body)))
    }

    /// A constructor to create a markdown notice.
    #[cfg(feature = "markdown")]
    pub fn notice_markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self::new(MessageType::Notice(NoticeMessageEventContent::markdown(body)))
    }

    /// Turns `self` into a reply to the given message.
    ///
    /// Takes the `body` / `formatted_body` (if any) in `self` for the main text and prepends a
    /// quoted version of `original_message`. Also sets the `in_reply_to` field inside `relates_to`.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/rich_reply.md"))]
    ///
    /// # Panics
    ///
    /// Panics if `self` has a `formatted_body` with a format other than HTML.
    #[track_caller]
    pub fn make_reply_to(mut self, original_message: &OriginalRoomMessageEvent) -> Self {
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

        self.relates_to = Some(Relation::Reply {
            in_reply_to: InReplyTo { event_id: original_message.event_id.to_owned() },
        });

        self
    }

    /// Create a new reply with the given message and optionally forwards the [`Relation::Thread`].
    ///
    /// If `message` is a text, an emote or a notice message, it is modified to include the rich
    /// reply fallback.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/rich_reply.md"))]
    #[cfg(feature = "unstable-msc3440")]
    pub fn reply(
        message: MessageType,
        original_message: &OriginalRoomMessageEvent,
        forward_thread: ForwardThread,
    ) -> Self {
        let make_reply = |body, formatted: Option<FormattedBody>| {
            reply::plain_and_formatted_reply_body(body, formatted.map(|f| f.body), original_message)
        };

        let msgtype = match message {
            MessageType::Text(TextMessageEventContent { body, formatted, .. }) => {
                let (body, html_body) = make_reply(body, formatted);
                MessageType::Text(TextMessageEventContent::html(body, html_body))
            }
            MessageType::Emote(EmoteMessageEventContent { body, formatted, .. }) => {
                let (body, html_body) = make_reply(body, formatted);
                MessageType::Emote(EmoteMessageEventContent::html(body, html_body))
            }
            MessageType::Notice(NoticeMessageEventContent { body, formatted, .. }) => {
                let (body, html_body) = make_reply(body, formatted);
                MessageType::Notice(NoticeMessageEventContent::html(body, html_body))
            }
            _ => message,
        };

        let relates_to = if let Some(Relation::Thread(Thread { event_id, .. })) = original_message
            .content
            .relates_to
            .as_ref()
            .filter(|_| forward_thread == ForwardThread::Yes)
        {
            Relation::Thread(Thread::reply(event_id.clone(), original_message.event_id.clone()))
        } else {
            Relation::Reply {
                in_reply_to: InReplyTo { event_id: original_message.event_id.clone() },
            }
        };

        Self { msgtype, relates_to: Some(relates_to) }
    }

    /// Create a new message for a thread that is optionally a reply.
    ///
    /// Looks for a [`Relation::Thread`] in `previous_message`. If it exists, a message for the same
    /// thread is created. If it doesn't, a new thread with `previous_message` as the root is
    /// created.
    ///
    /// If `message` is a text, an emote or a notice message, and this is a reply in the thread, it
    /// is modified to include the rich reply fallback.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/rich_reply.md"))]
    #[cfg(feature = "unstable-msc3440")]
    pub fn for_thread(
        message: MessageType,
        previous_message: &OriginalRoomMessageEvent,
        is_reply: ReplyInThread,
    ) -> Self {
        let make_reply = |body, formatted: Option<FormattedBody>| {
            reply::plain_and_formatted_reply_body(body, formatted.map(|f| f.body), previous_message)
        };

        let msgtype = if is_reply == ReplyInThread::Yes {
            // If this is a real reply, add the rich reply fallback.
            match message {
                MessageType::Text(TextMessageEventContent { body, formatted, .. }) => {
                    let (body, html_body) = make_reply(body, formatted);
                    MessageType::Text(TextMessageEventContent::html(body, html_body))
                }
                MessageType::Emote(EmoteMessageEventContent { body, formatted, .. }) => {
                    let (body, html_body) = make_reply(body, formatted);
                    MessageType::Emote(EmoteMessageEventContent::html(body, html_body))
                }
                MessageType::Notice(NoticeMessageEventContent { body, formatted, .. }) => {
                    let (body, html_body) = make_reply(body, formatted);
                    MessageType::Notice(NoticeMessageEventContent::html(body, html_body))
                }
                _ => message,
            }
        } else {
            message
        };

        let thread_root = if let Some(Relation::Thread(Thread { event_id, .. })) =
            &previous_message.content.relates_to
        {
            event_id.clone()
        } else {
            previous_message.event_id.clone()
        };

        Self {
            msgtype,
            relates_to: Some(Relation::Thread(Thread {
                event_id: thread_root,
                in_reply_to: InReplyTo { event_id: previous_message.event_id.clone() },
                is_falling_back: is_reply == ReplyInThread::No,
            })),
        }
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
    /// [tags and attributes]: https://spec.matrix.org/v1.2/client-server-api/#mroommessage-msgtypes
    /// [rich reply fallback]: https://spec.matrix.org/v1.2/client-server-api/#fallbacks-for-rich-replies
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
#[cfg(feature = "unstable-msc3440")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
pub enum ForwardThread {
    /// The thread relation in the original message is forwarded if it exists.
    ///
    /// This should be set if your client doesn't support threads (see [MSC3440]).
    ///
    /// [MSC3440]: https://github.com/matrix-org/matrix-spec-proposals/pull/3440
    Yes,

    /// Create a reply in the main conversation even if the original message is in a thread.
    ///
    /// This should be used if you client supports threads and you explicitly want that behavior.
    No,
}

/// Whether or not the message is a reply inside a thread.
#[cfg(feature = "unstable-msc3440")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
pub enum ReplyInThread {
    /// This is a reply.
    ///
    /// Create a proper reply _in_ the thread.
    Yes,

    /// This is not a reply.
    ///
    /// Create a regular message in the thread, with a reply fallback, according to [MSC3440].
    ///
    /// [MSC3440]: https://github.com/matrix-org/matrix-spec-proposals/pull/3440
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
    /// The `msgtype` and `body` are required fields as defined by [the `m.room.message` spec](https://spec.matrix.org/v1.2/client-server-api/#mroommessage).
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

/// Message event relationship.
#[derive(Clone, Debug)]
#[allow(clippy::manual_non_exhaustive)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum Relation {
    /// An `m.in_reply_to` relation indicating that the event is a reply to another event.
    Reply {
        /// Information about another message being replied to.
        in_reply_to: InReplyTo,
    },

    /// An event that replaces another event.
    #[cfg(feature = "unstable-msc2676")]
    Replacement(Replacement),

    /// An event that belongs to a thread.
    #[cfg(feature = "unstable-msc3440")]
    Thread(Thread),

    #[doc(hidden)]
    _Custom,
}

/// Information about the event a "rich reply" is replying to.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct InReplyTo {
    /// The event being replied to.
    pub event_id: OwnedEventId,
}

impl InReplyTo {
    /// Creates a new `InReplyTo` with the given event ID.
    pub fn new(event_id: OwnedEventId) -> Self {
        Self { event_id }
    }
}

/// The event this relation belongs to replaces another event.
#[derive(Clone, Debug)]
#[cfg(feature = "unstable-msc2676")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Replacement {
    /// The ID of the event being replaced.
    pub event_id: OwnedEventId,

    /// New content.
    pub new_content: Box<RoomMessageEventContent>,
}

#[cfg(feature = "unstable-msc2676")]
impl Replacement {
    /// Creates a new `Replacement` with the given event ID and new content.
    pub fn new(event_id: OwnedEventId, new_content: Box<RoomMessageEventContent>) -> Self {
        Self { event_id, new_content }
    }
}

/// The content of a thread relation.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg(feature = "unstable-msc3440")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Thread {
    /// The ID of the root message in the thread.
    pub event_id: OwnedEventId,

    /// A reply relation.
    ///
    /// If this event is a reply and belongs to a thread, this points to the message that is being
    /// replied to, and `is_falling_back` must be set to `false`.
    ///
    /// If this event is not a reply, this is used as a fallback mechanism for clients that do not
    /// support threads. This should point to the latest message-like event in the thread and
    /// `is_falling_back` must be set to `true`.
    pub in_reply_to: InReplyTo,

    /// Whether the `m.in_reply_to` field is a fallback for older clients or a genuine reply in a
    /// thread.
    pub is_falling_back: bool,
}

#[cfg(feature = "unstable-msc3440")]
impl Thread {
    /// Convenience method to create a regular `Thread` with the given event ID and latest
    /// message-like event ID.
    pub fn plain(event_id: OwnedEventId, latest_event_id: OwnedEventId) -> Self {
        Self { event_id, in_reply_to: InReplyTo::new(latest_event_id), is_falling_back: true }
    }

    /// Convenience method to create a reply `Thread` with the given event ID and replied-to event
    /// ID.
    pub fn reply(event_id: OwnedEventId, reply_to_event_id: OwnedEventId) -> Self {
        Self { event_id, in_reply_to: InReplyTo::new(reply_to_event_id), is_falling_back: false }
    }
}

/// The format for the formatted representation of a message body.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
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
        let body = body.as_ref();
        let mut html_body = String::new();

        pulldown_cmark::html::push_html(&mut html_body, pulldown_cmark::Parser::new(body));

        (html_body != format!("<p>{}</p>\n", body)).then(|| Self::html(html_body))
    }

    /// Sanitize this `FormattedBody` if its format is `MessageFormat::Html`.
    ///
    /// This removes any [tags and attributes] that are not listed in the Matrix specification.
    ///
    /// It can also optionally remove the [rich reply fallback].
    ///
    /// Returns the sanitized HTML if the format is `MessageFormat::Html`.
    ///
    /// [tags and attributes]: https://spec.matrix.org/v1.2/client-server-api/#mroommessage-msgtypes
    /// [rich reply fallback]: https://spec.matrix.org/v1.2/client-server-api/#fallbacks-for-rich-replies
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
