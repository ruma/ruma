//! Types for the [`m.room.message`] event.
//!
//! [`m.room.message`]: https://spec.matrix.org/latest/client-server-api/#mroommessage

use std::borrow::Cow;

use ruma_common::{
    serde::{JsonObject, Raw, StringEnum},
    OwnedEventId, RoomId,
};
#[cfg(feature = "html")]
use ruma_html::{sanitize_html, HtmlSanitizerMode, RemoveReplyFallback};
use ruma_macros::EventContent;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tracing::warn;

use self::reply::OriginalEventData;
#[cfg(feature = "html")]
use self::sanitize::remove_plain_reply_fallback;
use crate::{
    relation::{InReplyTo, Replacement, Thread},
    AnySyncTimelineEvent, Mentions, PrivOwnedStr,
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
mod reply;
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
        self,
        original_message: &OriginalRoomMessageEvent,
        forward_thread: ForwardThread,
        add_mentions: AddMentions,
    ) -> Self {
        self.without_relation().make_reply_to(original_message, forward_thread, add_mentions)
    }

    /// Turns `self` into a reply to the given raw event.
    ///
    /// Takes the `body` / `formatted_body` (if any) in `self` for the main text and prepends a
    /// quoted version of the `body` of `original_event` (if any). Also sets the `in_reply_to` field
    /// inside `relates_to`, and optionally the `rel_type` to `m.thread` if the
    /// `original_message is in a thread and thread forwarding is enabled.
    ///
    /// It is recommended to use [`Self::make_reply_to()`] for replies to `m.room.message` events,
    /// as the generated fallback is better for some `msgtype`s.
    ///
    /// Note that except for the panic below, this is infallible. Which means that if a field is
    /// missing when deserializing the data, the changes that require it will not be applied. It
    /// will still at least apply the `m.in_reply_to` relation to this content.
    ///
    /// # Panics
    ///
    /// Panics if `self` has a `formatted_body` with a format other than HTML.
    #[track_caller]
    pub fn make_reply_to_raw(
        self,
        original_event: &Raw<AnySyncTimelineEvent>,
        original_event_id: OwnedEventId,
        room_id: &RoomId,
        forward_thread: ForwardThread,
        add_mentions: AddMentions,
    ) -> Self {
        self.without_relation().make_reply_to_raw(
            original_event,
            original_event_id,
            room_id,
            forward_thread,
            add_mentions,
        )
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
        self,
        previous_message: &OriginalRoomMessageEvent,
        is_reply: ReplyWithinThread,
        add_mentions: AddMentions,
    ) -> Self {
        self.without_relation().make_for_thread(previous_message, is_reply, add_mentions)
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
    /// If the message that is replaced is a reply to another message, the latter should also be
    /// provided to be able to generate a rich reply fallback that takes the `body` /
    /// `formatted_body` (if any) in `self` for the main text and prepends a quoted version of
    /// `original_message`.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/rich_reply.md"))]
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
    pub fn make_replacement(
        self,
        metadata: impl Into<ReplacementMetadata>,
        replied_to_message: Option<&OriginalRoomMessageEvent>,
    ) -> Self {
        self.without_relation().make_replacement(metadata, replied_to_message)
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

    #[track_caller]
    fn add_reply_fallback(&mut self, original_event: OriginalEventData<'_>) {
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

        if let Some(f) = formatted {
            assert_eq!(
                f.format,
                MessageFormat::Html,
                "can't add reply fallback to non-HTML formatted messages"
            );

            let formatted_body = &mut f.body;

            (*body, *formatted_body) = reply::plain_and_formatted_reply_body(
                body.as_str(),
                (!formatted_body.is_empty()).then_some(formatted_body.as_str()),
                original_event,
            );
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
    use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, TagEnd};

    const OPTIONS: Options = Options::ENABLE_TABLES.union(Options::ENABLE_STRIKETHROUGH);

    let mut found_first_paragraph = false;
    let mut previous_event_was_text = false;

    let parser_events: Vec<_> = Parser::new_ext(text, OPTIONS)
        .map(|event| match event {
            Event::SoftBreak => Event::HardBreak,
            _ => event,
        })
        .collect();
    let has_markdown = parser_events.iter().any(|ref event| {
        // Numeric references should be replaced by their UTF-8 equivalent, so encountering a
        // non-borrowed string means that there is markdown syntax.
        let is_borrowed_text = matches!(event, Event::Text(CowStr::Borrowed(_)));

        if is_borrowed_text {
            if previous_event_was_text {
                // The text was split, so a character was likely removed, like in the case of
                // backslash escapes, or replaced by a static string, like for entity references, so
                // there is markdown syntax.
                return true;
            } else {
                previous_event_was_text = true;
            }
        } else {
            previous_event_was_text = false;
        }

        // A hard break happens when a newline is encountered, which is not necessarily markdown
        // syntax.
        let is_break = matches!(event, Event::HardBreak);

        // The parser always wraps the string into a paragraph, so the first paragraph should be
        // ignored, it is not due to markdown syntax.
        let is_first_paragraph_start = if matches!(event, Event::Start(Tag::Paragraph)) {
            if found_first_paragraph {
                false
            } else {
                found_first_paragraph = true;
                true
            }
        } else {
            false
        };
        let is_paragraph_end = matches!(event, Event::End(TagEnd::Paragraph));

        !is_borrowed_text && !is_break && !is_first_paragraph_start && !is_paragraph_end
    });

    if !has_markdown {
        return None;
    }

    let mut html_body = String::new();
    pulldown_cmark::html::push_html(&mut html_body, parser_events.into_iter());

    Some(html_body)
}

#[cfg(all(test, feature = "markdown"))]
mod tests {
    use assert_matches2::assert_matches;

    use super::parse_markdown;

    #[test]
    fn detect_markdown() {
        // Simple single-line text.
        let text = "Hello world.";
        assert_matches!(parse_markdown(text), None);

        // Simple double-line text.
        let text = "Hello\nworld.";
        assert_matches!(parse_markdown(text), None);

        // With new paragraph.
        let text = "Hello\n\nworld.";
        assert_matches!(parse_markdown(text), Some(_));

        // With tagged element.
        let text = "Hello **world**.";
        assert_matches!(parse_markdown(text), Some(_));

        // With backslash escapes.
        let text = r#"Hello \<world\>."#;
        assert_matches!(parse_markdown(text), Some(_));

        // With entity reference.
        let text = r#"Hello &lt;world&gt;."#;
        assert_matches!(parse_markdown(text), Some(_));

        // With numeric reference.
        let text = "Hello w&#8853;rld.";
        assert_matches!(parse_markdown(text), Some(_));
    }
}
