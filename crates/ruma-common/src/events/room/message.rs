//! Types for the [`m.room.message`] event.
//!
//! [`m.room.message`]: https://spec.matrix.org/v1.2/client-server-api/#mroommessage

use std::{borrow::Cow, fmt, time::Duration};

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value as JsonValue;

use super::{EncryptedFile, ImageInfo, MediaSource, ThumbnailInfo};
#[cfg(feature = "unstable-msc3246")]
use crate::events::audio::{AudioContent, AudioEventContent};
#[cfg(feature = "unstable-msc3551")]
use crate::events::file::{FileContent, FileContentInfo, FileEventContent};
#[cfg(feature = "unstable-msc3552")]
use crate::events::image::{ImageContent, ImageEventContent, ThumbnailContent};
#[cfg(feature = "unstable-msc3553")]
use crate::events::video::{VideoContent, VideoEventContent};
#[cfg(feature = "unstable-msc3245")]
use crate::events::voice::{VoiceContent, VoiceEventContent};
#[cfg(feature = "unstable-msc1767")]
use crate::events::{
    emote::EmoteEventContent,
    message::{MessageContent, MessageEventContent},
    notice::NoticeEventContent,
};
use crate::{
    events::key::verification::VerificationMethod,
    serde::{JsonObject, StringEnum},
    OwnedDeviceId, OwnedEventId, OwnedMxcUri, OwnedUserId, PrivOwnedStr,
};
#[cfg(feature = "unstable-msc3488")]
use crate::{
    events::location::{AssetContent, LocationContent, LocationEventContent},
    MilliSecondsSinceUnixEpoch,
};

mod content_serde;
pub mod feedback;
mod relation_serde;
mod reply;

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

    /// Creates a plain text reply to a message.
    ///
    /// This constructor requires an [`OriginalRoomMessageEvent`] since it creates a permalink to
    /// the previous message, for which the room ID is required. If you want to reply to an
    /// [`OriginalSyncRoomMessageEvent`], you have to convert it first by calling
    /// [`.into_full_event()`][crate::events::OriginalSyncMessageLikeEvent::into_full_event].
    pub fn text_reply_plain(
        reply: impl fmt::Display,
        original_message: &OriginalRoomMessageEvent,
    ) -> Self {
        let formatted: Option<&str> = None;
        let (body, html_body) =
            reply::plain_and_formatted_reply_body(reply, formatted, original_message);

        Self {
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo { event_id: original_message.event_id.to_owned() },
            }),
            ..Self::text_html(body, html_body)
        }
    }

    /// Creates a html text reply to a message.
    ///
    /// This constructor requires an [`OriginalRoomMessageEvent`] since it creates a permalink to
    /// the previous message, for which the room ID is required. If you want to reply to an
    /// [`OriginalSyncRoomMessageEvent`], you have to convert it first by calling
    /// [`.into_full_event()`][crate::events::OriginalSyncMessageLikeEvent::into_full_event].
    pub fn text_reply_html(
        reply: impl fmt::Display,
        html_reply: impl fmt::Display,
        original_message: &OriginalRoomMessageEvent,
    ) -> Self {
        let (body, html_body) =
            reply::plain_and_formatted_reply_body(reply, Some(html_reply), original_message);

        Self {
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo { event_id: original_message.event_id.clone() },
            }),
            ..Self::text_html(body, html_body)
        }
    }

    /// Creates a plain text notice reply to a message.
    ///
    /// This constructor requires an [`OriginalRoomMessageEvent`] since it creates a permalink to
    /// the previous message, for which the room ID is required. If you want to reply to an
    /// [`OriginalSyncRoomMessageEvent`], you have to convert it first by calling
    /// [`.into_full_event()`][crate::events::OriginalSyncMessageLikeEvent::into_full_event].
    pub fn notice_reply_plain(
        reply: impl fmt::Display,
        original_message: &OriginalRoomMessageEvent,
    ) -> Self {
        let formatted: Option<&str> = None;
        let (body, html_body) =
            reply::plain_and_formatted_reply_body(reply, formatted, original_message);

        Self {
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo { event_id: original_message.event_id.to_owned() },
            }),
            ..Self::notice_html(body, html_body)
        }
    }

    /// Creates a html text notice reply to a message.
    ///
    /// This constructor requires an [`OriginalRoomMessageEvent`] since it creates a permalink to
    /// the previous message, for which the room ID is required. If you want to reply to an
    /// [`OriginalSyncRoomMessageEvent`], you have to convert it first by calling
    /// [`.into_full_event()`][crate::events::OriginalSyncMessageLikeEvent::into_full_event].
    pub fn notice_reply_html(
        reply: impl fmt::Display,
        html_reply: impl fmt::Display,
        original_message: &OriginalRoomMessageEvent,
    ) -> Self {
        let (body, html_body) =
            reply::plain_and_formatted_reply_body(reply, Some(html_reply), original_message);

        Self {
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo { event_id: original_message.event_id.clone() },
            }),
            ..Self::notice_html(body, html_body)
        }
    }

    /// Create a new reply with the given message and optionally forwards the [`Relation::Thread`].
    ///
    /// If `message` is a text or notice message, it is modified to include the rich reply fallback.
    #[cfg(feature = "unstable-msc3440")]
    pub fn reply(
        message: MessageType,
        original_message: &OriginalRoomMessageEvent,
        forward_thread: ForwardThread,
    ) -> Self {
        let msgtype = match message {
            MessageType::Text(TextMessageEventContent { body, formatted, .. }) => {
                let (body, html_body) = reply::plain_and_formatted_reply_body(
                    body,
                    formatted.map(|f| f.body),
                    original_message,
                );

                MessageType::Text(TextMessageEventContent::html(body, html_body))
            }
            MessageType::Notice(NoticeMessageEventContent { body, formatted, .. }) => {
                let (body, html_body) = reply::plain_and_formatted_reply_body(
                    body,
                    formatted.map(|f| f.body),
                    original_message,
                );

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
    /// If `message` is a text or notice message, it is modified to include the rich reply fallback.
    #[cfg(feature = "unstable-msc3440")]
    pub fn for_thread(
        message: MessageType,
        previous_message: &OriginalRoomMessageEvent,
        is_reply: ReplyInThread,
    ) -> Self {
        let msgtype = match message {
            MessageType::Text(TextMessageEventContent { body, formatted, .. }) => {
                let (body, html_body) = reply::plain_and_formatted_reply_body(
                    body,
                    formatted.map(|f| f.body),
                    previous_message,
                );

                MessageType::Text(TextMessageEventContent::html(body, html_body))
            }
            MessageType::Notice(NoticeMessageEventContent { body, formatted, .. }) => {
                let (body, html_body) = reply::plain_and_formatted_reply_body(
                    body,
                    formatted.map(|f| f.body),
                    previous_message,
                );

                MessageType::Notice(NoticeMessageEventContent::html(body, html_body))
            }
            _ => message,
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
}

#[cfg(feature = "unstable-msc3246")]
impl From<AudioEventContent> for RoomMessageEventContent {
    fn from(content: AudioEventContent) -> Self {
        let AudioEventContent { message, file, audio, relates_to } = content;

        Self {
            msgtype: MessageType::Audio(AudioMessageEventContent::from_extensible_content(
                message, file, audio,
            )),
            relates_to,
        }
    }
}

#[cfg(feature = "unstable-msc1767")]
impl From<EmoteEventContent> for RoomMessageEventContent {
    fn from(content: EmoteEventContent) -> Self {
        let EmoteEventContent { message, relates_to, .. } = content;

        Self { msgtype: MessageType::Emote(message.into()), relates_to }
    }
}

#[cfg(feature = "unstable-msc3551")]
impl From<FileEventContent> for RoomMessageEventContent {
    fn from(content: FileEventContent) -> Self {
        let FileEventContent { message, file, relates_to } = content;

        Self {
            msgtype: MessageType::File(FileMessageEventContent::from_extensible_content(
                message, file,
            )),
            relates_to,
        }
    }
}

#[cfg(feature = "unstable-msc3552")]
impl From<ImageEventContent> for RoomMessageEventContent {
    fn from(content: ImageEventContent) -> Self {
        let ImageEventContent { message, file, image, thumbnail, caption, relates_to } = content;

        Self {
            msgtype: MessageType::Image(ImageMessageEventContent::from_extensible_content(
                message, file, image, thumbnail, caption,
            )),
            relates_to,
        }
    }
}

#[cfg(feature = "unstable-msc3488")]
impl From<LocationEventContent> for RoomMessageEventContent {
    fn from(content: LocationEventContent) -> Self {
        let LocationEventContent { message, location, asset, ts, relates_to } = content;

        Self {
            msgtype: MessageType::Location(LocationMessageEventContent::from_extensible_content(
                message, location, asset, ts,
            )),
            relates_to,
        }
    }
}

#[cfg(feature = "unstable-msc1767")]
impl From<MessageEventContent> for RoomMessageEventContent {
    fn from(content: MessageEventContent) -> Self {
        let MessageEventContent { message, relates_to, .. } = content;

        Self { msgtype: MessageType::Text(message.into()), relates_to }
    }
}

#[cfg(feature = "unstable-msc1767")]
impl From<NoticeEventContent> for RoomMessageEventContent {
    fn from(content: NoticeEventContent) -> Self {
        let NoticeEventContent { message, relates_to, .. } = content;

        Self { msgtype: MessageType::Notice(message.into()), relates_to }
    }
}

#[cfg(feature = "unstable-msc3553")]
impl From<VideoEventContent> for RoomMessageEventContent {
    fn from(content: VideoEventContent) -> Self {
        let VideoEventContent { message, file, video, thumbnail, caption, relates_to } = content;

        Self {
            msgtype: MessageType::Video(VideoMessageEventContent::from_extensible_content(
                message, file, video, thumbnail, caption,
            )),
            relates_to,
        }
    }
}

#[cfg(feature = "unstable-msc3245")]
impl From<VoiceEventContent> for RoomMessageEventContent {
    fn from(content: VoiceEventContent) -> Self {
        let VoiceEventContent { message, file, audio, voice, relates_to } = content;

        Self {
            msgtype: MessageType::Audio(AudioMessageEventContent::from_extensible_voice_content(
                message, file, audio, voice,
            )),
            relates_to,
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

/// The payload for an audio message.
///
/// With the `unstable-msc3246` feature, this type contains the transitional format of
/// [`AudioEventContent`] and with the `unstable-msc3245` feature, this type also contains the
/// transitional format of [`VoiceEventContent`]. See the documentation of the [`message`] module
/// for more information.
///
/// [`message`]: crate::events::message
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.audio")]
#[cfg_attr(
    feature = "unstable-msc3246",
    serde(from = "content_serde::AudioMessageEventContentDeHelper")
)]
pub struct AudioMessageEventContent {
    /// The textual representation of this message.
    pub body: String,

    /// The source of the audio clip.
    #[serde(flatten)]
    pub source: MediaSource,

    /// Metadata for the audio clip referred to in `source`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<AudioInfo>>,

    /// Extensible-event text representation of the message.
    ///
    /// If present, this should be preferred over the `body` field.
    #[cfg(feature = "unstable-msc3246")]
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub message: Option<MessageContent>,

    /// Extensible-event file content of the message.
    ///
    /// If present, this should be preferred over the `source` and `info` fields.
    #[cfg(feature = "unstable-msc3246")]
    #[serde(rename = "org.matrix.msc1767.file", skip_serializing_if = "Option::is_none")]
    pub file: Option<FileContent>,

    /// Extensible-event audio info of the message.
    ///
    /// If present, this should be preferred over the `info` field.
    #[cfg(feature = "unstable-msc3246")]
    #[serde(rename = "org.matrix.msc1767.audio", skip_serializing_if = "Option::is_none")]
    pub audio: Option<AudioContent>,

    /// Extensible-event voice flag of the message.
    ///
    /// If present, this should be represented as a voice message.
    #[cfg(feature = "unstable-msc3245")]
    #[serde(rename = "org.matrix.msc3245.voice", skip_serializing_if = "Option::is_none")]
    pub voice: Option<VoiceContent>,
}

impl AudioMessageEventContent {
    /// Creates a new non-encrypted `AudioMessageEventContent` with the given body, url and
    /// optional extra info.
    pub fn plain(body: String, url: OwnedMxcUri, info: Option<Box<AudioInfo>>) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3246")]
            message: Some(MessageContent::plain(body.clone())),
            #[cfg(feature = "unstable-msc3246")]
            file: Some(FileContent::plain(
                url.clone(),
                info.as_deref().map(|info| Box::new(info.into())),
            )),
            #[cfg(feature = "unstable-msc3246")]
            audio: Some(info.as_deref().map_or_else(AudioContent::default, Into::into)),
            #[cfg(feature = "unstable-msc3245")]
            voice: None,
            body,
            source: MediaSource::Plain(url),
            info,
        }
    }

    /// Creates a new encrypted `AudioMessageEventContent` with the given body and encrypted
    /// file.
    pub fn encrypted(body: String, file: EncryptedFile) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3246")]
            message: Some(MessageContent::plain(body.clone())),
            #[cfg(feature = "unstable-msc3246")]
            file: Some(FileContent::encrypted(file.url.clone(), (&file).into(), None)),
            #[cfg(feature = "unstable-msc3246")]
            audio: Some(AudioContent::default()),
            #[cfg(feature = "unstable-msc3245")]
            voice: None,
            body,
            source: MediaSource::Encrypted(Box::new(file)),
            info: None,
        }
    }

    /// Create a new `AudioMessageEventContent` with the given message, file info and audio info.
    #[cfg(feature = "unstable-msc3246")]
    pub fn from_extensible_content(
        message: MessageContent,
        file: FileContent,
        audio: AudioContent,
    ) -> Self {
        let body = if let Some(body) = message.find_plain() {
            body.to_owned()
        } else {
            message[0].body.clone()
        };
        let source = (&file).into();
        let info = AudioInfo::from_extensible_content(file.info.as_deref(), &audio).map(Box::new);

        Self {
            message: Some(message),
            file: Some(file),
            audio: Some(audio),
            #[cfg(feature = "unstable-msc3245")]
            voice: None,
            body,
            source,
            info,
        }
    }

    /// Create a new `AudioMessageEventContent` with the given message, file info, audio info and
    /// voice flag.
    #[cfg(feature = "unstable-msc3245")]
    pub fn from_extensible_voice_content(
        message: MessageContent,
        file: FileContent,
        audio: AudioContent,
        voice: VoiceContent,
    ) -> Self {
        let mut content = Self::from_extensible_content(message, file, audio);
        content.voice = Some(voice);
        content
    }
}

/// Metadata about an audio clip.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct AudioInfo {
    /// The duration of the audio in milliseconds.
    #[serde(
        with = "crate::serde::duration::opt_ms",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub duration: Option<Duration>,

    /// The mimetype of the audio, e.g. "audio/aac".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// The size of the audio clip in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<UInt>,
}

impl AudioInfo {
    /// Creates an empty `AudioInfo`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an `AudioInfo` from the given file info and audio info.
    #[cfg(feature = "unstable-msc3246")]
    pub fn from_extensible_content(
        file_info: Option<&FileContentInfo>,
        audio: &AudioContent,
    ) -> Option<Self> {
        if file_info.is_none() && audio.is_empty() {
            None
        } else {
            let (mimetype, size) = file_info
                .map(|info| (info.mimetype.to_owned(), info.size.to_owned()))
                .unwrap_or_default();
            let AudioContent { duration, .. } = audio;

            Some(Self { duration: duration.to_owned(), mimetype, size })
        }
    }
}

/// The payload for an emote message.
///
/// With the `unstable-msc1767` feature, this type contains the transitional format of
/// [`EmoteEventContent`]. See the documentation of the [`message`] module for more information.
///
/// [`message`]: crate::events::message
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.emote")]
pub struct EmoteMessageEventContent {
    /// The emote action to perform.
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

impl EmoteMessageEventContent {
    /// A convenience constructor to create a plain-text emote.
    pub fn plain(body: impl Into<String>) -> Self {
        let body = body.into();
        Self {
            #[cfg(feature = "unstable-msc1767")]
            message: Some(MessageContent::plain(body.clone())),
            body,
            formatted: None,
        }
    }

    /// A convenience constructor to create an html emote message.
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

    /// A convenience constructor to create a markdown emote.
    ///
    /// Returns an html emote message if some markdown formatting was detected, otherwise returns a
    /// plain-text emote.
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
impl From<MessageContent> for EmoteMessageEventContent {
    fn from(message: MessageContent) -> Self {
        let body = if let Some(body) = message.find_plain() { body } else { &message[0].body };
        let formatted = message.find_html().map(FormattedBody::html);

        Self { body: body.to_owned(), formatted, message: Some(message) }
    }
}

/// The payload for a file message.
///
/// With the `unstable-msc3551` feature, this type contains the transitional format of
/// [`FileEventContent`]. See the documentation of the [`message`] module for more information.
///
/// [`message`]: crate::events::message
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.file")]
#[cfg_attr(
    feature = "unstable-msc3551",
    serde(from = "content_serde::FileMessageEventContentDeHelper")
)]
pub struct FileMessageEventContent {
    /// A human-readable description of the file.
    ///
    /// This is recommended to be the filename of the original upload.
    pub body: String,

    /// The original filename of the uploaded file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,

    /// The source of the file.
    #[serde(flatten)]
    pub source: MediaSource,

    /// Metadata about the file referred to in `source`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<FileInfo>>,

    /// Extensible-event text representation of the message.
    ///
    /// If present, this should be preferred over the `body` field.
    #[cfg(feature = "unstable-msc3551")]
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub message: Option<MessageContent>,

    /// Extensible-event file content of the message.
    ///
    /// If present, this should be preferred over the `source` and `info` fields.
    #[cfg(feature = "unstable-msc3551")]
    #[serde(rename = "org.matrix.msc1767.file", skip_serializing_if = "Option::is_none")]
    pub file: Option<FileContent>,
}

impl FileMessageEventContent {
    /// Creates a new non-encrypted `FileMessageEventContent` with the given body, url and
    /// optional extra info.
    pub fn plain(body: String, url: OwnedMxcUri, info: Option<Box<FileInfo>>) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3551")]
            message: Some(MessageContent::plain(body.clone())),
            #[cfg(feature = "unstable-msc3551")]
            file: Some(FileContent::plain(
                url.clone(),
                info.as_deref().map(|info| Box::new(info.into())),
            )),
            body,
            filename: None,
            source: MediaSource::Plain(url),
            info,
        }
    }

    /// Creates a new encrypted `FileMessageEventContent` with the given body and encrypted
    /// file.
    pub fn encrypted(body: String, file: EncryptedFile) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3551")]
            message: Some(MessageContent::plain(body.clone())),
            #[cfg(feature = "unstable-msc3551")]
            file: Some(FileContent::encrypted(file.url.clone(), (&file).into(), None)),
            body,
            filename: None,
            source: MediaSource::Encrypted(Box::new(file)),
            info: None,
        }
    }

    /// Create a new `FileMessageEventContent` with the given message and file info.
    #[cfg(feature = "unstable-msc3551")]
    pub fn from_extensible_content(message: MessageContent, file: FileContent) -> Self {
        let body = if let Some(body) = message.find_plain() {
            body.to_owned()
        } else {
            message[0].body.clone()
        };
        let filename = file.info.as_deref().and_then(|info| info.name.clone());
        let info = file.info.as_deref().map(|info| Box::new(info.into()));
        let source = (&file).into();

        Self { message: Some(message), file: Some(file), body, filename, source, info }
    }
}

/// Metadata about a file.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct FileInfo {
    /// The mimetype of the file, e.g. "application/msword".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// The size of the file in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<UInt>,

    /// Metadata about the image referred to in `thumbnail_source`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_info: Option<Box<ThumbnailInfo>>,

    /// The source of the thumbnail of the file.
    #[serde(
        flatten,
        with = "super::thumbnail_source_serde",
        skip_serializing_if = "Option::is_none"
    )]
    pub thumbnail_source: Option<MediaSource>,
}

impl FileInfo {
    /// Creates an empty `FileInfo`.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(feature = "unstable-msc3551")]
impl From<&FileContentInfo> for FileInfo {
    fn from(info: &FileContentInfo) -> Self {
        let FileContentInfo { mimetype, size, .. } = info;
        Self { mimetype: mimetype.to_owned(), size: size.to_owned(), ..Default::default() }
    }
}

/// The payload for an image message.
///
/// With the `unstable-msc3552` feature, this type contains the transitional format of
/// [`ImageEventContent`]. See the documentation of the [`message`] module for more information.
///
/// [`message`]: crate::events::message
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.image")]
#[cfg_attr(
    feature = "unstable-msc3552",
    serde(from = "content_serde::ImageMessageEventContentDeHelper")
)]
pub struct ImageMessageEventContent {
    /// A textual representation of the image.
    ///
    /// Could be the alt text of the image, the filename of the image, or some kind of content
    /// description for accessibility e.g. "image attachment".
    pub body: String,

    /// The source of the image.
    #[serde(flatten)]
    pub source: MediaSource,

    /// Metadata about the image referred to in `source`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<ImageInfo>>,

    /// Extensible-event text representation of the message.
    ///
    /// If present, this should be preferred over the `body` field.
    #[cfg(feature = "unstable-msc3552")]
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub message: Option<MessageContent>,

    /// Extensible-event file content of the message.
    ///
    /// If present, this should be preferred over the `source` and `info` fields.
    #[cfg(feature = "unstable-msc3552")]
    #[serde(rename = "org.matrix.msc1767.file", skip_serializing_if = "Option::is_none")]
    pub file: Option<FileContent>,

    /// Extensible-event image info of the message.
    ///
    /// If present, this should be preferred over the `info` field.
    #[cfg(feature = "unstable-msc3552")]
    #[serde(rename = "org.matrix.msc1767.image", skip_serializing_if = "Option::is_none")]
    pub image: Option<Box<ImageContent>>,

    /// Extensible-event thumbnails of the message.
    ///
    /// If present, this should be preferred over the `info` field.
    #[cfg(feature = "unstable-msc3552")]
    #[serde(rename = "org.matrix.msc1767.thumbnail", skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<Vec<ThumbnailContent>>,

    /// Extensible-event captions of the message.
    #[cfg(feature = "unstable-msc3552")]
    #[serde(
        rename = "org.matrix.msc1767.caption",
        with = "crate::events::message::content_serde::as_vec",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub caption: Option<MessageContent>,
}

impl ImageMessageEventContent {
    /// Creates a new non-encrypted `ImageMessageEventContent` with the given body, url and
    /// optional extra info.
    pub fn plain(body: String, url: OwnedMxcUri, info: Option<Box<ImageInfo>>) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3552")]
            message: Some(MessageContent::plain(body.clone())),
            #[cfg(feature = "unstable-msc3552")]
            file: Some(FileContent::plain(
                url.clone(),
                info.as_deref().map(|info| Box::new(info.into())),
            )),
            #[cfg(feature = "unstable-msc3552")]
            image: Some(Box::new(info.as_deref().map_or_else(ImageContent::default, Into::into))),
            #[cfg(feature = "unstable-msc3552")]
            thumbnail: info
                .as_deref()
                .and_then(|info| {
                    ThumbnailContent::from_room_message_content(
                        info.thumbnail_source.as_ref(),
                        info.thumbnail_info.as_deref(),
                    )
                })
                .map(|thumbnail| vec![thumbnail]),
            #[cfg(feature = "unstable-msc3552")]
            caption: None,
            body,
            source: MediaSource::Plain(url),
            info,
        }
    }

    /// Creates a new encrypted `ImageMessageEventContent` with the given body and encrypted
    /// file.
    pub fn encrypted(body: String, file: EncryptedFile) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3552")]
            message: Some(MessageContent::plain(body.clone())),
            #[cfg(feature = "unstable-msc3552")]
            file: Some(FileContent::encrypted(file.url.clone(), (&file).into(), None)),
            #[cfg(feature = "unstable-msc3552")]
            image: Some(Box::new(ImageContent::default())),
            #[cfg(feature = "unstable-msc3552")]
            thumbnail: None,
            #[cfg(feature = "unstable-msc3552")]
            caption: None,
            body,
            source: MediaSource::Encrypted(Box::new(file)),
            info: None,
        }
    }

    /// Create a new `ImageMessageEventContent` with the given message, file info, image info,
    /// thumbnails and captions.
    #[cfg(feature = "unstable-msc3552")]
    pub fn from_extensible_content(
        message: MessageContent,
        file: FileContent,
        image: Box<ImageContent>,
        thumbnail: Vec<ThumbnailContent>,
        caption: Option<MessageContent>,
    ) -> Self {
        let body = if let Some(body) = message.find_plain() {
            body.to_owned()
        } else {
            message[0].body.clone()
        };
        let source = (&file).into();
        let info = ImageInfo::from_extensible_content(file.info.as_deref(), &image, &thumbnail)
            .map(Box::new);
        let thumbnail = if thumbnail.is_empty() { None } else { Some(thumbnail) };

        Self {
            message: Some(message),
            file: Some(file),
            image: Some(image),
            thumbnail,
            caption,
            body,
            source,
            info,
        }
    }
}

/// The payload for a location message.
///
/// With the `unstable-msc3488` feature, this type contains the transitional format of
/// [`LocationEventContent`]. See the documentation of the [`message`] module for more information.
///
/// [`message`]: crate::events::message
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.location")]
#[cfg_attr(
    feature = "unstable-msc3488",
    serde(from = "content_serde::LocationMessageEventContentDeHelper")
)]
pub struct LocationMessageEventContent {
    /// A description of the location e.g. "Big Ben, London, UK", or some kind of content
    /// description for accessibility, e.g. "location attachment".
    pub body: String,

    /// A geo URI representing the location.
    pub geo_uri: String,

    /// Info about the location being represented.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<LocationInfo>>,

    /// Extensible-event text representation of the message.
    ///
    /// If present, this should be preferred over the `body` field.
    #[cfg(feature = "unstable-msc3488")]
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub message: Option<MessageContent>,

    /// Extensible-event location info of the message.
    ///
    /// If present, this should be preferred over the `geo_uri` field.
    #[cfg(feature = "unstable-msc3488")]
    #[serde(rename = "org.matrix.msc3488.location", skip_serializing_if = "Option::is_none")]
    pub location: Option<LocationContent>,

    /// Extensible-event asset this message refers to.
    #[cfg(feature = "unstable-msc3488")]
    #[serde(rename = "org.matrix.msc3488.asset", skip_serializing_if = "Option::is_none")]
    pub asset: Option<AssetContent>,

    /// Extensible-event timestamp this message refers to.
    #[cfg(feature = "unstable-msc3488")]
    #[serde(rename = "org.matrix.msc3488.ts", skip_serializing_if = "Option::is_none")]
    pub ts: Option<MilliSecondsSinceUnixEpoch>,
}

impl LocationMessageEventContent {
    /// Creates a new `LocationMessageEventContent` with the given body and geo URI.
    pub fn new(body: String, geo_uri: String) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3488")]
            message: Some(MessageContent::plain(body.clone())),
            #[cfg(feature = "unstable-msc3488")]
            location: Some(LocationContent::new(geo_uri.clone())),
            #[cfg(feature = "unstable-msc3488")]
            asset: None,
            #[cfg(feature = "unstable-msc3488")]
            ts: None,
            body,
            geo_uri,
            info: None,
        }
    }

    /// Create a new `LocationMessageEventContent` with the given message, location info, asset and
    /// timestamp.
    #[cfg(feature = "unstable-msc3488")]
    pub fn from_extensible_content(
        message: MessageContent,
        location: LocationContent,
        asset: AssetContent,
        ts: Option<MilliSecondsSinceUnixEpoch>,
    ) -> Self {
        let body = if let Some(body) = message.find_plain() {
            body.to_owned()
        } else {
            message[0].body.clone()
        };
        let geo_uri = location.uri.clone();

        Self {
            message: Some(message),
            location: Some(location),
            asset: Some(asset),
            ts,
            body,
            geo_uri,
            info: None,
        }
    }
}

/// Thumbnail info associated with a location.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct LocationInfo {
    /// The source of a thumbnail of the location.
    #[serde(
        flatten,
        with = "super::thumbnail_source_serde",
        skip_serializing_if = "Option::is_none"
    )]
    pub thumbnail_source: Option<MediaSource>,

    /// Metadata about the image referred to in `thumbnail_source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_info: Option<Box<ThumbnailInfo>>,
}

impl LocationInfo {
    /// Creates an empty `LocationInfo`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// The payload for a notice message.
///
/// With the `unstable-msc1767` feature, this type contains the transitional format of
/// [`NoticeEventContent`]. See the documentation of the [`message`] module for more information.
///
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

/// The payload for a server notice message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.server_notice")]
pub struct ServerNoticeMessageEventContent {
    /// A human-readable description of the notice.
    pub body: String,

    /// The type of notice being represented.
    pub server_notice_type: ServerNoticeType,

    /// A URI giving a contact method for the server administrator.
    ///
    /// Required if the notice type is `m.server_notice.usage_limit_reached`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_contact: Option<String>,

    /// The kind of usage limit the server has exceeded.
    ///
    /// Required if the notice type is `m.server_notice.usage_limit_reached`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_type: Option<LimitType>,
}

impl ServerNoticeMessageEventContent {
    /// Creates a new `ServerNoticeMessageEventContent` with the given body and notice type.
    pub fn new(body: String, server_notice_type: ServerNoticeType) -> Self {
        Self { body, server_notice_type, admin_contact: None, limit_type: None }
    }
}

/// Types of server notices.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[non_exhaustive]
pub enum ServerNoticeType {
    /// The server has exceeded some limit which requires the server administrator to intervene.
    #[ruma_enum(rename = "m.server_notice.usage_limit_reached")]
    UsageLimitReached,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl ServerNoticeType {
    /// Creates a string slice from this `ServerNoticeType`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

/// Types of usage limits.
///
/// This type can hold an arbitrary string. To build this with a custom value, convert it from a
/// string with `::from() / .into()`. To check for formats that are not available as a documented
/// variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum LimitType {
    /// The server's number of active users in the last 30 days has exceeded the maximum.
    ///
    /// New connections are being refused by the server. What defines "active" is left as an
    /// implementation detail, however servers are encouraged to treat syncing users as "active".
    MonthlyActiveUser,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl LimitType {
    /// Creates a string slice from this `LimitType`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

/// The format for the formatted representation of a message body.
///
/// This type can hold an arbitrary string. To build this with a custom value, convert it from a
/// string with `::from() / .into()`. To check for formats that are not available as a documented
/// variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[non_exhaustive]
pub enum MessageFormat {
    /// HTML.
    #[ruma_enum(rename = "org.matrix.custom.html")]
    Html,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl MessageFormat {
    /// Creates a string slice from this `MessageFormat`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
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
}

/// The payload for a text message.
///
/// With the `unstable-msc1767` feature, this type contains the transitional format of
/// [`MessageEventContent`]. See the documentation of the [`message`] module for more information.
///
/// [`message`]: crate::events::message
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.text")]
pub struct TextMessageEventContent {
    /// The body of the message.
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

impl TextMessageEventContent {
    /// A convenience constructor to create a plain text message.
    pub fn plain(body: impl Into<String>) -> Self {
        let body = body.into();
        Self {
            #[cfg(feature = "unstable-msc1767")]
            message: Some(MessageContent::plain(body.clone())),
            body,
            formatted: None,
        }
    }

    /// A convenience constructor to create an HTML message.
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

#[cfg(feature = "unstable-msc1767")]
impl From<MessageContent> for TextMessageEventContent {
    fn from(message: MessageContent) -> Self {
        let body = if let Some(body) = message.find_plain() { body } else { &message[0].body };
        let formatted = message.find_html().map(FormattedBody::html);

        Self { body: body.to_owned(), formatted, message: Some(message) }
    }
}

/// The payload for a video message.
///
/// With the `unstable-msc3553` feature, this type contains the transitional format of
/// [`VideoEventContent`]. See the documentation of the [`message`] module for more information.
///
/// [`message`]: crate::events::message
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.video")]
#[cfg_attr(
    feature = "unstable-msc3553",
    serde(from = "content_serde::VideoMessageEventContentDeHelper")
)]
pub struct VideoMessageEventContent {
    /// A description of the video, e.g. "Gangnam Style", or some kind of content description for
    /// accessibility, e.g. "video attachment".
    pub body: String,

    /// The source of the video clip.
    #[serde(flatten)]
    pub source: MediaSource,

    /// Metadata about the video clip referred to in `source`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<VideoInfo>>,

    /// Extensible-event text representation of the message.
    ///
    /// If present, this should be preferred over the `body` field.
    #[cfg(feature = "unstable-msc3553")]
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub message: Option<MessageContent>,

    /// Extensible-event file content of the message.
    ///
    /// If present, this should be preferred over the `source` and `info` fields.
    #[cfg(feature = "unstable-msc3553")]
    #[serde(rename = "org.matrix.msc1767.file", skip_serializing_if = "Option::is_none")]
    pub file: Option<FileContent>,

    /// Extensible-event video info of the message.
    ///
    /// If present, this should be preferred over the `info` field.
    #[cfg(feature = "unstable-msc3553")]
    #[serde(rename = "org.matrix.msc1767.video", skip_serializing_if = "Option::is_none")]
    pub video: Option<Box<VideoContent>>,

    /// Extensible-event thumbnails of the message.
    ///
    /// If present, this should be preferred over the `info` field.
    #[cfg(feature = "unstable-msc3553")]
    #[serde(rename = "org.matrix.msc1767.thumbnail", skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<Vec<ThumbnailContent>>,

    /// Extensible-event captions of the message.
    #[cfg(feature = "unstable-msc3553")]
    #[serde(
        rename = "org.matrix.msc1767.caption",
        with = "crate::events::message::content_serde::as_vec",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub caption: Option<MessageContent>,
}

impl VideoMessageEventContent {
    /// Creates a new non-encrypted `VideoMessageEventContent` with the given body, url and
    /// optional extra info.
    pub fn plain(body: String, url: OwnedMxcUri, info: Option<Box<VideoInfo>>) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3553")]
            message: Some(MessageContent::plain(body.clone())),
            #[cfg(feature = "unstable-msc3553")]
            file: Some(FileContent::plain(
                url.clone(),
                info.as_deref().map(|info| Box::new(info.into())),
            )),
            #[cfg(feature = "unstable-msc3553")]
            video: Some(Box::new(info.as_deref().map_or_else(VideoContent::default, Into::into))),
            #[cfg(feature = "unstable-msc3553")]
            thumbnail: info
                .as_deref()
                .and_then(|info| {
                    ThumbnailContent::from_room_message_content(
                        info.thumbnail_source.as_ref(),
                        info.thumbnail_info.as_deref(),
                    )
                })
                .map(|thumbnail| vec![thumbnail]),
            #[cfg(feature = "unstable-msc3553")]
            caption: None,
            body,
            source: MediaSource::Plain(url),
            info,
        }
    }

    /// Creates a new encrypted `VideoMessageEventContent` with the given body and encrypted
    /// file.
    pub fn encrypted(body: String, file: EncryptedFile) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3553")]
            message: Some(MessageContent::plain(body.clone())),
            #[cfg(feature = "unstable-msc3553")]
            file: Some(FileContent::encrypted(file.url.clone(), (&file).into(), None)),
            #[cfg(feature = "unstable-msc3553")]
            video: Some(Box::new(VideoContent::default())),
            #[cfg(feature = "unstable-msc3553")]
            thumbnail: None,
            #[cfg(feature = "unstable-msc3553")]
            caption: None,
            body,
            source: MediaSource::Encrypted(Box::new(file)),
            info: None,
        }
    }

    /// Create a new `VideoMessageEventContent` with the given message, file info, video info,
    /// thumbnails and captions.
    #[cfg(feature = "unstable-msc3553")]
    pub fn from_extensible_content(
        message: MessageContent,
        file: FileContent,
        video: Box<VideoContent>,
        thumbnail: Vec<ThumbnailContent>,
        caption: Option<MessageContent>,
    ) -> Self {
        let body = if let Some(body) = message.find_plain() {
            body.to_owned()
        } else {
            message[0].body.clone()
        };
        let source = (&file).into();
        let info = VideoInfo::from_extensible_content(file.info.as_deref(), &video, &thumbnail)
            .map(Box::new);
        let thumbnail = if thumbnail.is_empty() { None } else { Some(thumbnail) };

        Self {
            message: Some(message),
            file: Some(file),
            video: Some(video),
            thumbnail,
            caption,
            body,
            source,
            info,
        }
    }
}

/// Metadata about a video.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct VideoInfo {
    /// The duration of the video in milliseconds.
    #[serde(
        with = "crate::serde::duration::opt_ms",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub duration: Option<Duration>,

    /// The height of the video in pixels.
    #[serde(rename = "h", skip_serializing_if = "Option::is_none")]
    pub height: Option<UInt>,

    /// The width of the video in pixels.
    #[serde(rename = "w", skip_serializing_if = "Option::is_none")]
    pub width: Option<UInt>,

    /// The mimetype of the video, e.g. "video/mp4".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// The size of the video in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<UInt>,

    /// Metadata about the image referred to in `thumbnail_source`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_info: Option<Box<ThumbnailInfo>>,

    /// The source of the thumbnail of the video clip.
    #[serde(
        flatten,
        with = "super::thumbnail_source_serde",
        skip_serializing_if = "Option::is_none"
    )]
    pub thumbnail_source: Option<MediaSource>,

    /// The [BlurHash](https://blurha.sh) for this video.
    ///
    /// This uses the unstable prefix in
    /// [MSC2448](https://github.com/matrix-org/matrix-spec-proposals/pull/2448).
    #[cfg(feature = "unstable-msc2448")]
    #[serde(rename = "xyz.amorgan.blurhash", skip_serializing_if = "Option::is_none")]
    pub blurhash: Option<String>,
}

impl VideoInfo {
    /// Creates an empty `VideoInfo`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a `VideoInfo` from the given file info, video info and thumbnail.
    #[cfg(feature = "unstable-msc3553")]
    pub fn from_extensible_content(
        file_info: Option<&FileContentInfo>,
        video: &VideoContent,
        thumbnail: &[ThumbnailContent],
    ) -> Option<Self> {
        if file_info.is_none() && video.is_empty() && thumbnail.is_empty() {
            None
        } else {
            let (mimetype, size) = file_info
                .map(|info| (info.mimetype.to_owned(), info.size.to_owned()))
                .unwrap_or_default();
            let VideoContent { duration, height, width } = video.to_owned();
            let (thumbnail_source, thumbnail_info) = thumbnail
                .get(0)
                .map(|thumbnail| {
                    let source = (&thumbnail.file).into();
                    let info = ThumbnailInfo::from_extensible_content(
                        thumbnail.file.info.as_deref(),
                        thumbnail.image.as_deref(),
                    )
                    .map(Box::new);
                    (Some(source), info)
                })
                .unwrap_or_default();

            Some(Self {
                duration,
                height,
                width,
                mimetype,
                size,
                thumbnail_info,
                thumbnail_source,
                #[cfg(feature = "unstable-msc2448")]
                blurhash: None,
            })
        }
    }
}

/// The payload for a key verification request message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.key.verification.request")]
pub struct KeyVerificationRequestEventContent {
    /// A fallback message to alert users that their client does not support the key verification
    /// framework.
    pub body: String,

    /// The verification methods supported by the sender.
    pub methods: Vec<VerificationMethod>,

    /// The device ID which is initiating the request.
    pub from_device: OwnedDeviceId,

    /// The user ID which should receive the request.
    ///
    /// Users should only respond to verification requests if they are named in this field. Users
    /// who are not named in this field and who did not send this event should ignore all other
    /// events that have a `m.reference` relationship with this event.
    pub to: OwnedUserId,
}

impl KeyVerificationRequestEventContent {
    /// Creates a new `KeyVerificationRequestEventContent` with the given body, method, device
    /// and user ID.
    pub fn new(
        body: String,
        methods: Vec<VerificationMethod>,
        from_device: OwnedDeviceId,
        to: OwnedUserId,
    ) -> Self {
        Self { body, methods, from_device, to }
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
