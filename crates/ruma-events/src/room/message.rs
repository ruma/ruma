//! Types for the [`m.room.message`] event.
//!
//! [`m.room.message`]: https://spec.matrix.org/v1.2/client-server-api/#mroommessage

use std::{borrow::Cow, fmt};

use js_int::UInt;
use ruma_events_macros::EventContent;
use ruma_identifiers::{DeviceId, EventId, MxcUri, UserId};
use ruma_serde::{JsonObject, StringEnum};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value as JsonValue;

use super::{EncryptedFile, ImageInfo, ThumbnailInfo};
use crate::{key::verification::VerificationMethod, PrivOwnedStr};

mod content_serde;
pub mod feedback;
mod relation_serde;
mod reply;

pub use reply::ReplyBaseEvent;

/// The content of an `m.room.message` event.
///
/// This event is used when sending messages in a room.
///
/// Messages are not limited to be text.
#[derive(Clone, Debug, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.message", kind = Message)]
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
    pub fn text_reply_plain(
        reply: impl fmt::Display,
        original_message: &impl ReplyBaseEvent,
    ) -> Self {
        let quoted = reply::get_plain_quote_fallback(original_message);

        let body = format!("{}\n\n{}", quoted, reply);

        Self {
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo { event_id: original_message.event_id().to_owned() },
            }),
            ..Self::text_plain(body)
        }
    }

    /// Creates a html text reply to a message.
    ///
    /// Different from `text_reply_plain`, this constructor requires specifically a
    /// [`RoomMessageEvent`] since it creates a permalink to the previous message, for which the
    /// room ID is required. If you want to reply to a [`SyncRoomMessageEvent`], you have to convert
    /// it first by calling [`.into_full_event()`][crate::SyncMessageEvent::into_full_event].
    pub fn text_reply_html(
        reply: impl fmt::Display,
        html_reply: impl fmt::Display,
        original_message: &RoomMessageEvent,
    ) -> Self {
        let quoted = reply::get_plain_quote_fallback(original_message);
        let quoted_html = reply::get_html_quote_fallback(original_message);

        let body = format!("{}\n\n{}", quoted, reply);
        let html_body = format!("{}\n\n{}", quoted_html, html_reply);

        Self {
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo { event_id: original_message.event_id.clone() },
            }),
            ..Self::text_html(body, html_body)
        }
    }

    /// Creates a plain text notice reply to a message.
    pub fn notice_reply_plain(
        reply: impl fmt::Display,
        original_message: &impl ReplyBaseEvent,
    ) -> Self {
        let quoted = reply::get_plain_quote_fallback(original_message);

        let body = format!("{}\n\n{}", quoted, reply);
        Self {
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo { event_id: original_message.event_id().to_owned() },
            }),
            ..Self::notice_plain(body)
        }
    }

    /// Creates a html text notice reply to a message.
    ///
    /// Different from `notice_reply_plain`, this constructor requires specifically a
    /// [`RoomMessageEvent`] since it creates a permalink to the previous message, for which the
    /// room ID is required. If you want to reply to a [`SyncRoomMessageEvent`], you have to convert
    /// it first by calling [`.into_full_event()`][crate::SyncMessageEvent::into_full_event].
    pub fn notice_reply_html(
        reply: impl fmt::Display,
        html_reply: impl fmt::Display,
        original_message: &RoomMessageEvent,
    ) -> Self {
        let quoted = reply::get_plain_quote_fallback(original_message);
        let quoted_html = reply::get_html_quote_fallback(original_message);

        let body = format!("{}\n\n{}", quoted, reply);
        let html_body = format!("{}\n\n{}", quoted_html, html_reply);

        Self {
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo { event_id: original_message.event_id.clone() },
            }),
            ..Self::notice_html(body, html_body)
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
    /// Creates a new `MessageType` with the given `msgtype` string and data.
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
///
/// Currently used for replies and editing (message replacement).
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

    #[doc(hidden)]
    _Custom,
}

/// Information about the event a "rich reply" is replying to.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct InReplyTo {
    /// The event being replied to.
    pub event_id: Box<EventId>,
}

impl InReplyTo {
    /// Creates a new `InReplyTo` with the given event ID.
    pub fn new(event_id: Box<EventId>) -> Self {
        Self { event_id }
    }
}

/// The event this relation belongs to replaces another event.
#[derive(Clone, Debug)]
#[cfg(feature = "unstable-msc2676")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Replacement {
    /// The ID of the event being replacing.
    pub event_id: Box<EventId>,

    /// New content.
    pub new_content: Box<RoomMessageEventContent>,
}

#[cfg(feature = "unstable-msc2676")]
impl Replacement {
    /// Creates a new `Replacement` with the given event ID and new content.
    pub fn new(event_id: Box<EventId>, new_content: Box<RoomMessageEventContent>) -> Self {
        Self { event_id, new_content }
    }
}

/// The payload for an audio message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.audio")]
pub struct AudioMessageEventContent {
    /// The textual representation of this message.
    pub body: String,

    /// The URL to the audio clip.
    ///
    /// Required if the file is unencrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Box<MxcUri>>,

    /// Information on the encrypted audio clip.
    ///
    /// Required if the audio clip is encrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<Box<EncryptedFile>>,

    /// Metadata for the audio clip referred to in `url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<AudioInfo>>,
}

impl AudioMessageEventContent {
    /// Creates a new non-encrypted `RoomAudioMessageEventContent` with the given body, url and
    /// optional extra info.
    pub fn plain(body: String, url: Box<MxcUri>, info: Option<Box<AudioInfo>>) -> Self {
        Self { body, url: Some(url), info, file: None }
    }

    /// Creates a new encrypted `RoomAudioMessageEventContent` with the given body and encrypted
    /// file.
    pub fn encrypted(body: String, file: EncryptedFile) -> Self {
        Self { body, url: None, info: None, file: Some(Box::new(file)) }
    }
}

/// Metadata about an audio clip.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct AudioInfo {
    /// The duration of the audio in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<UInt>,

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
}

/// The payload for an emote message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.emote")]
pub struct EmoteMessageEventContent {
    /// The emote action to perform.
    pub body: String,

    /// Formatted form of the message `body`.
    #[serde(flatten)]
    pub formatted: Option<FormattedBody>,
}

impl EmoteMessageEventContent {
    /// A convenience constructor to create a plain-text emote.
    pub fn plain(body: impl Into<String>) -> Self {
        Self { body: body.into(), formatted: None }
    }

    /// A convenience constructor to create an html emote message.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self { formatted: Some(FormattedBody::html(html_body)), ..Self::plain(body) }
    }

    /// A convenience constructor to create a markdown emote.
    ///
    /// Returns an html emote message if some markdown formatting was detected, otherwise returns a
    /// plain-text emote.
    #[cfg(feature = "markdown")]
    pub fn markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self { formatted: FormattedBody::markdown(&body), ..Self::plain(body) }
    }
}

/// The payload for a file message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.file")]
pub struct FileMessageEventContent {
    /// A human-readable description of the file.
    ///
    /// This is recommended to be the filename of the original upload.
    pub body: String,

    /// The original filename of the uploaded file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,

    /// The URL to the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Box<MxcUri>>,

    /// Information on the encrypted file.
    ///
    /// Required if file is encrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<Box<EncryptedFile>>,

    /// Metadata about the file referred to in `url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<FileInfo>>,
}

impl FileMessageEventContent {
    /// Creates a new non-encrypted `RoomFileMessageEventContent` with the given body, url and
    /// optional extra info.
    pub fn plain(body: String, url: Box<MxcUri>, info: Option<Box<FileInfo>>) -> Self {
        Self { body, filename: None, url: Some(url), info, file: None }
    }

    /// Creates a new encrypted `RoomFileMessageEventContent` with the given body and encrypted
    /// file.
    pub fn encrypted(body: String, file: EncryptedFile) -> Self {
        Self { body, filename: None, url: None, info: None, file: Some(Box::new(file)) }
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

    /// Metadata about the image referred to in `thumbnail_url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_info: Option<Box<ThumbnailInfo>>,

    /// The URL to the thumbnail of the file.
    ///
    /// Only present if the thumbnail is unencrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<Box<MxcUri>>,

    /// Information on the encrypted thumbnail file.
    ///
    /// Only present if the thumbnail is encrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_file: Option<Box<EncryptedFile>>,
}

impl FileInfo {
    /// Creates an empty `FileInfo`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// The payload for an image message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.image")]
pub struct ImageMessageEventContent {
    /// A textual representation of the image.
    ///
    /// Could be the alt text of the image, the filename of the image, or some kind of content
    /// description for accessibility e.g. "image attachment".
    pub body: String,

    /// The URL to the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Box<MxcUri>>,

    /// Information on the encrypted image.
    ///
    /// Required if image is encrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<Box<EncryptedFile>>,

    /// Metadata about the image referred to in `url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<ImageInfo>>,
}

impl ImageMessageEventContent {
    /// Creates a new non-encrypted `RoomImageMessageEventContent` with the given body, url and
    /// optional extra info.
    pub fn plain(body: String, url: Box<MxcUri>, info: Option<Box<ImageInfo>>) -> Self {
        Self { body, url: Some(url), info, file: None }
    }

    /// Creates a new encrypted `RoomImageMessageEventContent` with the given body and encrypted
    /// file.
    pub fn encrypted(body: String, file: EncryptedFile) -> Self {
        Self { body, url: None, info: None, file: Some(Box::new(file)) }
    }
}

/// The payload for a location message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.location")]
pub struct LocationMessageEventContent {
    /// A description of the location e.g. "Big Ben, London, UK", or some kind of content
    /// description for accessibility, e.g. "location attachment".
    pub body: String,

    /// A geo URI representing the location.
    pub geo_uri: String,

    /// Info about the location being represented.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<LocationInfo>>,
}

impl LocationMessageEventContent {
    /// Creates a new `RoomLocationMessageEventContent` with the given body and geo URI.
    pub fn new(body: String, geo_uri: String) -> Self {
        Self { body, geo_uri, info: None }
    }
}

/// Thumbnail info associated with a location.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct LocationInfo {
    /// The URL to a thumbnail of the location being represented.
    ///
    /// Only present if the thumbnail is unencrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<Box<MxcUri>>,

    /// Information on an encrypted thumbnail of the location being represented.
    ///
    /// Only present if the thumbnail is encrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_file: Option<Box<EncryptedFile>>,

    /// Metadata about the image referred to in `thumbnail_url` or `thumbnail_file`.
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
        Self { body: body.into(), formatted: None }
    }

    /// A convenience constructor to create an html notice.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self { formatted: Some(FormattedBody::html(html_body)), ..Self::plain(body) }
    }

    /// A convenience constructor to create a markdown notice.
    ///
    /// Returns an html notice if some markdown formatting was detected, otherwise returns a plain
    /// text notice.
    #[cfg(feature = "markdown")]
    pub fn markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self { formatted: FormattedBody::markdown(&body), ..Self::plain(body) }
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
    /// Creates a new `RoomServerNoticeMessageEventContent` with the given body and notice type.
    pub fn new(body: String, server_notice_type: ServerNoticeType) -> Self {
        Self { body, server_notice_type, admin_contact: None, limit_type: None }
    }
}

/// Types of server notices.
///
/// This type can hold an arbitrary string. To check for formats that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
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
/// This type can hold an arbitrary string. To check for formats that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
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
/// This type can hold an arbitrary string. To check for formats that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
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

    /// Creates a new HTML-formatted message body by parsing the markdown in `body`.
    ///
    /// Returns `None` if no markdown formatting was found.
    #[cfg(feature = "markdown")]
    pub fn markdown(body: impl AsRef<str>) -> Option<Self> {
        let body = body.as_ref();
        let mut html_body = String::new();

        pulldown_cmark::html::push_html(&mut html_body, pulldown_cmark::Parser::new(body));

        (html_body != format!("<p>{}</p>\n", body)).then(|| Self::html(html_body))
    }
}

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
}

impl TextMessageEventContent {
    /// A convenience constructor to create a plain text message.
    pub fn plain(body: impl Into<String>) -> Self {
        Self { body: body.into(), formatted: None }
    }

    /// A convenience constructor to create an html message.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self { formatted: Some(FormattedBody::html(html_body)), ..Self::plain(body) }
    }

    /// A convenience constructor to create a markdown message.
    ///
    /// Returns an html message if some markdown formatting was detected, otherwise returns a plain
    /// text message.
    #[cfg(feature = "markdown")]
    pub fn markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self { formatted: FormattedBody::markdown(&body), ..Self::plain(body) }
    }
}

/// The payload for a video message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.video")]
pub struct VideoMessageEventContent {
    /// A description of the video, e.g. "Gangnam Style", or some kind of content description for
    /// accessibility, e.g. "video attachment".
    pub body: String,

    /// The URL to the video clip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Box<MxcUri>>,

    /// Information on the encrypted video clip.
    ///
    /// Required if video clip is encrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<Box<EncryptedFile>>,

    /// Metadata about the video clip referred to in `url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<VideoInfo>>,
}

impl VideoMessageEventContent {
    /// Creates a new non-encrypted `RoomVideoMessageEventContent` with the given body, url and
    /// optional extra info.
    pub fn plain(body: String, url: Box<MxcUri>, info: Option<Box<VideoInfo>>) -> Self {
        Self { body, url: Some(url), info, file: None }
    }

    /// Creates a new encrypted `RoomVideoMessageEventContent` with the given body and encrypted
    /// file.
    pub fn encrypted(body: String, file: EncryptedFile) -> Self {
        Self { body, url: None, info: None, file: Some(Box::new(file)) }
    }
}

/// Metadata about a video.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct VideoInfo {
    /// The duration of the video in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<UInt>,

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

    /// Metadata about an image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_info: Option<Box<ThumbnailInfo>>,

    /// The URL to an image thumbnail of the video clip.
    ///
    /// Only present if the thumbnail is unencrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<Box<MxcUri>>,

    /// Information on the encrypted thumbnail file.
    ///
    /// Only present if the thumbnail is encrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_file: Option<Box<EncryptedFile>>,

    /// The [BlurHash](https://blurha.sh) for this video.
    ///
    /// This uses the unstable prefix in
    /// [MSC2448](https://github.com/matrix-org/matrix-doc/pull/2448).
    #[cfg(feature = "unstable-msc2448")]
    #[serde(rename = "xyz.amorgan.blurhash", skip_serializing_if = "Option::is_none")]
    pub blurhash: Option<String>,
}

impl VideoInfo {
    /// Creates an empty `VideoInfo`.
    pub fn new() -> Self {
        Self::default()
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
    pub from_device: Box<DeviceId>,

    /// The user ID which should receive the request.
    ///
    /// Users should only respond to verification requests if they are named in this field. Users
    /// who are not named in this field and who did not send this event should ignore all other
    /// events that have a `m.reference` relationship with this event.
    pub to: Box<UserId>,
}

impl KeyVerificationRequestEventContent {
    /// Creates a new `RoomKeyVerificationRequestEventContent` with the given body, method, device
    /// and user ID.
    pub fn new(
        body: String,
        methods: Vec<VerificationMethod>,
        from_device: Box<DeviceId>,
        to: Box<UserId>,
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

#[cfg(test)]
mod tests {
    use matches::assert_matches;
    use ruma_identifiers::event_id;
    use serde_json::{from_value as from_json_value, json};

    use super::{InReplyTo, MessageType, Relation, RoomMessageEventContent};

    #[test]
    fn deserialize_reply() {
        let ev_id = event_id!("$1598361704261elfgc:localhost");

        let json = json!({
            "msgtype": "m.text",
            "body": "<text msg>",
            "m.relates_to": {
                "m.in_reply_to": {
                    "event_id": ev_id,
                },
            },
        });

        assert_matches!(
            from_json_value::<RoomMessageEventContent>(json).unwrap(),
            RoomMessageEventContent {
                msgtype: MessageType::Text(_),
                relates_to: Some(Relation::Reply { in_reply_to: InReplyTo { event_id } }),
            } if event_id == ev_id
        );
    }
}
