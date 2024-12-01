//! Types for the [`m.room.message`] event.
//!
//! [`m.room.message`]: https://spec.matrix.org/latest/client-server-api/#mroommessage

use std::borrow::Cow;

use as_variant::as_variant;
use ruma_common::{
    serde::{JsonObject, StringEnum},
    EventId, OwnedEventId, UserId,
};
#[cfg(feature = "html")]
use ruma_html::{sanitize_html, HtmlSanitizerMode, RemoveReplyFallback};
use ruma_macros::EventContent;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tracing::warn;

#[cfg(feature = "html")]
use self::sanitize::remove_plain_reply_fallback;
use crate::{
    relation::{InReplyTo, Replacement, Thread},
    Mentions, PrivOwnedStr,
};

mod audio;
mod content_serde;
mod emote;
mod file;
mod image;
mod key_verification_request;
mod location;
mod media_caption;
mod notice;
mod relation;
pub(crate) mod relation_serde;
pub mod sanitize;
mod server_notice;
mod text;
#[cfg(feature = "unstable-msc4095")]
mod url_preview;
mod video;
mod without_relation;

#[cfg(feature = "unstable-msc3245-v1-compat")]
pub use self::audio::{
    UnstableAmplitude, UnstableAudioDetailsContentBlock, UnstableVoiceContentBlock,
};
#[cfg(feature = "unstable-msc4095")]
pub use self::url_preview::UrlPreview;
pub use self::{
    audio::{AudioInfo, AudioMessageEventContent},
    emote::EmoteMessageEventContent,
    file::{FileInfo, FileMessageEventContent},
    image::ImageMessageEventContent,
    key_verification_request::KeyVerificationRequestEventContent,
    location::{LocationInfo, LocationMessageEventContent},
    notice::NoticeMessageEventContent,
    relation::{Relation, RelationWithoutReplacement},
    relation_serde::deserialize_relation,
    server_notice::{LimitType, ServerNoticeMessageEventContent, ServerNoticeType},
    text::TextMessageEventContent,
    video::{VideoInfo, VideoMessageEventContent},
    without_relation::RoomMessageEventContentWithoutRelation,
};

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

    /// Information about [related messages].
    ///
    /// [related messages]: https://spec.matrix.org/latest/client-server-api/#forming-relationships-between-events
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Relation<RoomMessageEventContentWithoutRelation>>,

    /// The [mentions] of this event.
    ///
    /// This should always be set to avoid triggering the legacy mention push rules. It is
    /// recommended to use [`Self::set_mentions()`] to set this field, that will take care of
    /// populating the fields correctly if this is a replacement.
    ///
    /// [mentions]: https://spec.matrix.org/latest/client-server-api/#user-and-room-mentions
    #[serde(rename = "m.mentions", skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Mentions>,
}

impl RoomMessageEventContent {
    /// Create a `RoomMessageEventContent` with the given `MessageType`.
    pub fn new(msgtype: MessageType) -> Self {
        Self { msgtype, relates_to: None, mentions: None }
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

    /// A constructor to create a plain text emote.
    pub fn emote_plain(body: impl Into<String>) -> Self {
        Self::new(MessageType::emote_plain(body))
    }

    /// A constructor to create an html emote.
    pub fn emote_html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self::new(MessageType::emote_html(body, html_body))
    }

    /// A constructor to create a markdown emote.
    #[cfg(feature = "markdown")]
    pub fn emote_markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self::new(MessageType::emote_markdown(body))
    }

    /// Turns `self` into a [rich reply] to the message using the given metadata.
    ///
    /// Sets the `in_reply_to` field inside `relates_to`, and optionally the `rel_type` to
    /// `m.thread` if the metadata has a `thread` and `ForwardThread::Yes` is used.
    ///
    /// If `AddMentions::Yes` is used, the `sender` in the metadata is added as a user mention.
    ///
    /// [rich reply]: https://spec.matrix.org/latest/client-server-api/#rich-replies
    #[track_caller]
    pub fn make_reply_to<'a>(
        self,
        metadata: impl Into<ReplyMetadata<'a>>,
        forward_thread: ForwardThread,
        add_mentions: AddMentions,
    ) -> Self {
        self.without_relation().make_reply_to(metadata, forward_thread, add_mentions)
    }

    /// Turns `self` into a new message for a [thread], that is optionally a reply.
    ///
    /// Looks for the `thread` in the given metadata. If it exists, this message will be in the same
    /// thread. If it doesn't, a new thread is created with the `event_id` in the metadata as the
    /// root.
    ///
    /// It also sets the `in_reply_to` field inside `relates_to` to point the `event_id`
    /// in the metadata. If `ReplyWithinThread::Yes` is used, the metadata should be constructed
    /// from the event to make a reply to, otherwise it should be constructed from the latest
    /// event in the thread.
    ///
    /// If `AddMentions::Yes` is used, the `sender` in the metadata is added as a user mention.
    ///
    /// [thread]: https://spec.matrix.org/latest/client-server-api/#threading
    pub fn make_for_thread<'a>(
        self,
        metadata: impl Into<ReplyMetadata<'a>>,
        is_reply: ReplyWithinThread,
        add_mentions: AddMentions,
    ) -> Self {
        self.without_relation().make_for_thread(metadata, is_reply, add_mentions)
    }

    /// Turns `self` into a [replacement] (or edit) for a given message.
    ///
    /// The first argument after `self` can be `&OriginalRoomMessageEvent` or
    /// `&OriginalSyncRoomMessageEvent` if you don't want to create `ReplacementMetadata` separately
    /// before calling this function.
    ///
    /// This takes the content and sets it in `m.new_content`, and modifies the `content` to include
    /// a fallback.
    ///
    /// If this message contains [`Mentions`], they are copied into `m.new_content` to keep the same
    /// mentions, but the ones in `content` are filtered with the ones in the
    /// [`ReplacementMetadata`] so only new mentions will trigger a notification.
    ///
    /// # Panics
    ///
    /// Panics if `self` has a `formatted_body` with a format other than HTML.
    ///
    /// [replacement]: https://spec.matrix.org/latest/client-server-api/#event-replacements
    #[track_caller]
    pub fn make_replacement(self, metadata: impl Into<ReplacementMetadata>) -> Self {
        self.without_relation().make_replacement(metadata)
    }

    /// Set the [mentions] of this event.
    ///
    /// If this event is a replacement, it will update the mentions both in the `content` and the
    /// `m.new_content` so only new mentions will trigger a notification. As such, this needs to be
    /// called after [`Self::make_replacement()`].
    ///
    /// It is not recommended to call this method after one that sets mentions automatically, like
    /// [`Self::make_reply_to()`] as these will be overwritten. [`Self::add_mentions()`] should be
    /// used instead.
    ///
    /// [mentions]: https://spec.matrix.org/latest/client-server-api/#user-and-room-mentions
    #[deprecated = "Call add_mentions before adding the relation instead."]
    pub fn set_mentions(mut self, mentions: Mentions) -> Self {
        if let Some(Relation::Replacement(replacement)) = &mut self.relates_to {
            let old_mentions = &replacement.new_content.mentions;

            let new_mentions = if let Some(old_mentions) = old_mentions {
                let mut new_mentions = Mentions::new();

                new_mentions.user_ids = mentions
                    .user_ids
                    .iter()
                    .filter(|u| !old_mentions.user_ids.contains(*u))
                    .cloned()
                    .collect();

                new_mentions.room = mentions.room && !old_mentions.room;

                new_mentions
            } else {
                mentions.clone()
            };

            replacement.new_content.mentions = Some(mentions);
            self.mentions = Some(new_mentions);
        } else {
            self.mentions = Some(mentions);
        }

        self
    }

    /// Add the given [mentions] to this event.
    ///
    /// If no [`Mentions`] was set on this events, this sets it. Otherwise, this updates the current
    /// mentions by extending the previous `user_ids` with the new ones, and applies a logical OR to
    /// the values of `room`.
    ///
    /// This should be called before methods that add a relation, like [`Self::make_reply_to()`] and
    /// [`Self::make_replacement()`], for the mentions to be correctly set.
    ///
    /// [mentions]: https://spec.matrix.org/latest/client-server-api/#user-and-room-mentions
    pub fn add_mentions(mut self, mentions: Mentions) -> Self {
        self.mentions.get_or_insert_with(Mentions::new).add(mentions);
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

    /// Apply the given new content from a [`Replacement`] to this message.
    pub fn apply_replacement(&mut self, new_content: RoomMessageEventContentWithoutRelation) {
        let RoomMessageEventContentWithoutRelation { msgtype, mentions } = new_content;
        self.msgtype = msgtype;
        self.mentions = mentions;
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
    #[cfg(feature = "html")]
    pub fn sanitize(
        &mut self,
        mode: HtmlSanitizerMode,
        remove_reply_fallback: RemoveReplyFallback,
    ) {
        let remove_reply_fallback = if matches!(self.relates_to, Some(Relation::Reply { .. })) {
            remove_reply_fallback
        } else {
            RemoveReplyFallback::No
        };

        self.msgtype.sanitize(mode, remove_reply_fallback);
    }

    fn without_relation(self) -> RoomMessageEventContentWithoutRelation {
        if self.relates_to.is_some() {
            warn!("Overwriting existing relates_to value");
        }

        self.into()
    }

    /// Get the thread relation from this content, if any.
    fn thread(&self) -> Option<&Thread> {
        self.relates_to.as_ref().and_then(|relates_to| as_variant!(relates_to, Relation::Thread))
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

/// Whether or not to add intentional [`Mentions`] when sending a reply.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
pub enum AddMentions {
    /// Add automatic intentional mentions to the reply.
    ///
    /// Set this if your client supports intentional mentions.
    ///
    /// The sender of the original event will be added to the mentions of this message.
    Yes,

    /// Do not add intentional mentions to the reply.
    ///
    /// Set this if your client does not support intentional mentions.
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

    /// A constructor to create a plain text emote.
    pub fn emote_plain(body: impl Into<String>) -> Self {
        Self::Emote(EmoteMessageEventContent::plain(body))
    }

    /// A constructor to create an html emote.
    pub fn emote_html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self::Emote(EmoteMessageEventContent::html(body, html_body))
    }

    /// A constructor to create a markdown emote.
    #[cfg(feature = "markdown")]
    pub fn emote_markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self::Emote(EmoteMessageEventContent::markdown(body))
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

    /// Sanitize this message.
    ///
    /// If this message contains HTML, this removes the [tags and attributes] that are not listed in
    /// the Matrix specification.
    ///
    /// It can also optionally remove the [rich reply fallback] from the plain text and HTML
    /// message. Note that you should be sure that the message is a reply, as there is no way to
    /// differentiate plain text reply fallbacks and markdown quotes.
    ///
    /// This method is only effective on text, notice and emote messages.
    ///
    /// [tags and attributes]: https://spec.matrix.org/latest/client-server-api/#mroommessage-msgtypes
    /// [rich reply fallback]: https://spec.matrix.org/latest/client-server-api/#fallbacks-for-rich-replies
    #[cfg(feature = "html")]
    pub fn sanitize(
        &mut self,
        mode: HtmlSanitizerMode,
        remove_reply_fallback: RemoveReplyFallback,
    ) {
        if let MessageType::Emote(EmoteMessageEventContent { body, formatted, .. })
        | MessageType::Notice(NoticeMessageEventContent { body, formatted, .. })
        | MessageType::Text(TextMessageEventContent { body, formatted, .. }) = self
        {
            if let Some(formatted) = formatted {
                formatted.sanitize_html(mode, remove_reply_fallback);
            }
            // This is a false positive, see <https://github.com/rust-lang/rust-clippy/issues/12444>
            #[allow(clippy::assigning_clones)]
            if remove_reply_fallback == RemoveReplyFallback::Yes {
                *body = remove_plain_reply_fallback(body).to_owned();
            }
        }
    }

    fn make_replacement_body(&mut self) {
        let empty_formatted_body = || FormattedBody::html(String::new());

        let (body, formatted) = {
            match self {
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

/// Metadata about an event to be replaced.
///
/// To be used with [`RoomMessageEventContent::make_replacement`].
#[derive(Debug)]
pub struct ReplacementMetadata {
    event_id: OwnedEventId,
    mentions: Option<Mentions>,
}

impl ReplacementMetadata {
    /// Creates a new `ReplacementMetadata` with the given event ID and mentions.
    pub fn new(event_id: OwnedEventId, mentions: Option<Mentions>) -> Self {
        Self { event_id, mentions }
    }
}

impl From<&OriginalRoomMessageEvent> for ReplacementMetadata {
    fn from(value: &OriginalRoomMessageEvent) -> Self {
        ReplacementMetadata::new(value.event_id.to_owned(), value.content.mentions.clone())
    }
}

impl From<&OriginalSyncRoomMessageEvent> for ReplacementMetadata {
    fn from(value: &OriginalSyncRoomMessageEvent) -> Self {
        ReplacementMetadata::new(value.event_id.to_owned(), value.content.mentions.clone())
    }
}

/// Metadata about an event to reply to or to add to a thread.
///
/// To be used with [`RoomMessageEventContent::make_reply_to`] or
/// [`RoomMessageEventContent::make_for_thread`].
#[derive(Clone, Copy, Debug)]
pub struct ReplyMetadata<'a> {
    /// The event ID of the event to reply to.
    event_id: &'a EventId,
    /// The sender of the event to reply to.
    sender: &'a UserId,
    /// The `m.thread` relation of the event to reply to, if any.
    thread: Option<&'a Thread>,
}

impl<'a> ReplyMetadata<'a> {
    /// Creates a new `ReplyMetadata` with the given event ID, sender and thread relation.
    pub fn new(event_id: &'a EventId, sender: &'a UserId, thread: Option<&'a Thread>) -> Self {
        Self { event_id, sender, thread }
    }
}

impl<'a> From<&'a OriginalRoomMessageEvent> for ReplyMetadata<'a> {
    fn from(value: &'a OriginalRoomMessageEvent) -> Self {
        ReplyMetadata::new(&value.event_id, &value.sender, value.content.thread())
    }
}

impl<'a> From<&'a OriginalSyncRoomMessageEvent> for ReplyMetadata<'a> {
    fn from(value: &'a OriginalSyncRoomMessageEvent) -> Self {
        ReplyMetadata::new(&value.event_id, &value.sender, value.content.thread())
    }
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
    #[cfg(feature = "html")]
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
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

    const OPTIONS: Options = Options::ENABLE_TABLES.union(Options::ENABLE_STRIKETHROUGH);

    let parser_events: Vec<_> = Parser::new_ext(text, OPTIONS)
        .map(|event| match event {
            Event::SoftBreak => Event::HardBreak,
            _ => event,
        })
        .collect();

    // Text that does not contain markdown syntax is always inline because when we encounter several
    // blocks we convert them to HTML. Inline text is always wrapped by a single paragraph.
    let first_event_is_paragraph_start =
        parser_events.first().is_some_and(|event| matches!(event, Event::Start(Tag::Paragraph)));
    let last_event_is_paragraph_end =
        parser_events.last().is_some_and(|event| matches!(event, Event::End(TagEnd::Paragraph)));
    let mut is_inline = first_event_is_paragraph_start && last_event_is_paragraph_end;
    let mut has_markdown = !is_inline;

    if !has_markdown {
        // Check whether the events contain other blocks and whether they contain inline markdown
        // syntax.
        let mut pos = 0;

        for event in parser_events.iter().skip(1) {
            match event {
                Event::Text(s) => {
                    // If the string does not contain markdown, the only modification that should
                    // happen is that newlines are converted to hardbreaks. It means that we should
                    // find all the other characters from the original string in the text events.
                    // Let's check that by walking the original string.
                    if text[pos..].starts_with(s.as_ref()) {
                        pos += s.len();
                        continue;
                    }
                }
                Event::HardBreak => {
                    // A hard break happens when a newline is encountered, which is not necessarily
                    // markdown syntax. Skip the newline in the original string for the walking
                    // above to work.
                    if text[pos..].starts_with("\r\n") {
                        pos += 2;
                        continue;
                    } else if text[pos..].starts_with(['\r', '\n']) {
                        pos += 1;
                        continue;
                    }
                }
                // A paragraph end is fine because we would detect markdown from the paragraph
                // start.
                Event::End(TagEnd::Paragraph) => continue,
                // Any other event means there is markdown syntax.
                Event::Start(tag) => {
                    is_inline &= !is_block_tag(tag);
                }
                _ => {}
            }

            has_markdown = true;

            // Stop when we also know that there are several blocks.
            if !is_inline {
                break;
            }
        }

        // If we are not at the end of the string, some characters were removed.
        has_markdown |= pos != text.len();
    }

    // If the string does not contain markdown, don't generate HTML.
    if !has_markdown {
        return None;
    }

    let mut events_iter = parser_events.into_iter();

    // If the content is inline, remove the wrapping paragraph, as instructed by the Matrix spec.
    if is_inline {
        events_iter.next();
        events_iter.next_back();
    }

    let mut html_body = String::new();
    pulldown_cmark::html::push_html(&mut html_body, events_iter);

    Some(html_body)
}

/// Whether the given tag is a block HTML element.
#[cfg(feature = "markdown")]
fn is_block_tag(tag: &pulldown_cmark::Tag<'_>) -> bool {
    use pulldown_cmark::Tag;

    matches!(
        tag,
        Tag::Paragraph
            | Tag::Heading { .. }
            | Tag::BlockQuote(_)
            | Tag::CodeBlock(_)
            | Tag::HtmlBlock
            | Tag::List(_)
            | Tag::FootnoteDefinition(_)
            | Tag::Table(_)
    )
}

#[cfg(all(test, feature = "markdown"))]
mod tests {
    use super::parse_markdown;

    #[test]
    fn detect_markdown() {
        // Simple single-line text.
        let text = "Hello world.";
        assert_eq!(parse_markdown(text), None);

        // Simple double-line text.
        let text = "Hello\nworld.";
        assert_eq!(parse_markdown(text), None);

        // With new paragraph.
        let text = "Hello\n\nworld.";
        assert_eq!(parse_markdown(text).as_deref(), Some("<p>Hello</p>\n<p>world.</p>\n"));

        // With heading and paragraph.
        let text = "## Hello\n\nworld.";
        assert_eq!(parse_markdown(text).as_deref(), Some("<h2>Hello</h2>\n<p>world.</p>\n"));

        // With paragraph and code block.
        let text = "Hello\n\n```\nworld.\n```";
        assert_eq!(
            parse_markdown(text).as_deref(),
            Some("<p>Hello</p>\n<pre><code>world.\n</code></pre>\n")
        );

        // With tagged element.
        let text = "Hello **world**.";
        assert_eq!(parse_markdown(text).as_deref(), Some("Hello <strong>world</strong>."));

        // Containing backslash escapes.
        let text = r#"Hello \<world\>."#;
        assert_eq!(parse_markdown(text).as_deref(), Some("Hello &lt;world&gt;."));

        // Starting with backslash escape.
        let text = r#"\> Hello world."#;
        assert_eq!(parse_markdown(text).as_deref(), Some("&gt; Hello world."));

        // With entity reference.
        let text = r#"Hello &lt;world&gt;."#;
        assert_eq!(parse_markdown(text).as_deref(), Some("Hello &lt;world&gt;."));

        // With numeric reference.
        let text = "Hello w&#8853;rld.";
        assert_eq!(parse_markdown(text).as_deref(), Some("Hello w⊕rld."));
    }

    #[test]
    fn detect_commonmark() {
        // Examples from the CommonMark spec.

        let text = r#"\!\"\#\$\%\&\'\(\)\*\+\,\-\.\/\:\;\<\=\>\?\@\[\\\]\^\_\`\{\|\}\~"#;
        assert_eq!(
            parse_markdown(text).as_deref(),
            Some(r##"!"#$%&amp;'()*+,-./:;&lt;=&gt;?@[\]^_`{|}~"##)
        );

        let text = r#"\→\A\a\ \3\φ\«"#;
        assert_eq!(parse_markdown(text).as_deref(), None);

        let text = r#"\*not emphasized*"#;
        assert_eq!(parse_markdown(text).as_deref(), Some("*not emphasized*"));

        let text = r#"\<br/> not a tag"#;
        assert_eq!(parse_markdown(text).as_deref(), Some("&lt;br/&gt; not a tag"));

        let text = r#"\[not a link](/foo)"#;
        assert_eq!(parse_markdown(text).as_deref(), Some("[not a link](/foo)"));

        let text = r#"\`not code`"#;
        assert_eq!(parse_markdown(text).as_deref(), Some("`not code`"));

        let text = r#"1\. not a list"#;
        assert_eq!(parse_markdown(text).as_deref(), Some("1. not a list"));

        let text = r#"\* not a list"#;
        assert_eq!(parse_markdown(text).as_deref(), Some("* not a list"));

        let text = r#"\# not a heading"#;
        assert_eq!(parse_markdown(text).as_deref(), Some("# not a heading"));

        let text = r#"\[foo]: /url "not a reference""#;
        assert_eq!(parse_markdown(text).as_deref(), Some(r#"[foo]: /url "not a reference""#));

        let text = r#"\&ouml; not a character entity"#;
        assert_eq!(parse_markdown(text).as_deref(), Some("&amp;ouml; not a character entity"));

        let text = r#"\\*emphasis*"#;
        assert_eq!(parse_markdown(text).as_deref(), Some(r#"\<em>emphasis</em>"#));

        let text = "foo\\\nbar";
        assert_eq!(parse_markdown(text).as_deref(), Some("foo<br />\nbar"));

        let text = " ***\n  ***\n   ***";
        assert_eq!(parse_markdown(text).as_deref(), Some("<hr />\n<hr />\n<hr />\n"));

        let text = "Foo\n***\nbar";
        assert_eq!(parse_markdown(text).as_deref(), Some("<p>Foo</p>\n<hr />\n<p>bar</p>\n"));

        let text = "</div>\n*foo*";
        assert_eq!(parse_markdown(text).as_deref(), Some("</div>\n*foo*"));

        let text = "<div>\n*foo*\n\n*bar*";
        assert_eq!(parse_markdown(text).as_deref(), Some("<div>\n*foo*\n<p><em>bar</em></p>\n"));

        let text = "aaa\nbbb\n\nccc\nddd";
        assert_eq!(
            parse_markdown(text).as_deref(),
            Some("<p>aaa<br />\nbbb</p>\n<p>ccc<br />\nddd</p>\n")
        );

        let text = "  aaa\n bbb";
        assert_eq!(parse_markdown(text).as_deref(), Some("aaa<br />\nbbb"));

        let text = "aaa\n             bbb\n                                       ccc";
        assert_eq!(parse_markdown(text).as_deref(), Some("aaa<br />\nbbb<br />\nccc"));

        let text = "aaa     \nbbb     ";
        assert_eq!(parse_markdown(text).as_deref(), Some("aaa<br />\nbbb"));
    }
}
