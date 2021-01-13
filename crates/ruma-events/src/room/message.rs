//! Types for the *m.room.message* event.

use std::borrow::Cow;

use indoc::formatdoc;
use js_int::UInt;
use ruma_events_macros::EventContent;
use ruma_identifiers::MxcUri;
#[cfg(feature = "unstable-pre-spec")]
use ruma_identifiers::{DeviceIdBox, UserId};
use ruma_serde::StringEnum;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[cfg(feature = "unstable-pre-spec")]
use super::relationships::{Annotation, Reference, RelationJsonRepr, Replacement};
use super::{relationships::RelatesToJsonRepr, EncryptedFile, ImageInfo, ThumbnailInfo};
#[cfg(feature = "unstable-pre-spec")]
use crate::key::verification::VerificationMethod;

// FIXME: Do we want to keep re-exporting this?
pub use super::relationships::InReplyTo;

mod content_serde;
pub mod feedback;

type JsonObject = serde_json::Map<String, JsonValue>;

/// This event is used when sending messages in a room.
///
/// Messages are not limited to be text.
pub type MessageEvent = crate::MessageEvent<MessageEventContent>;

/// The payload for `MessageEvent`.
#[derive(Clone, Debug, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.message", kind = Message)]
pub struct MessageEventContent {
    /// A key which identifies the type of message being sent.
    ///
    /// This also holds the specific content of each message.
    #[serde(flatten)]
    pub msgtype: MessageType,

    /// Information about related messages for
    /// [rich replies](https://matrix.org/docs/spec/client_server/r0.6.1#rich-replies).
    #[serde(rename = "m.relates_to", skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Relation>,

    /// New content of an edited message.
    ///
    /// This should only be set if `relates_to` is `Some(Relation::Replacement(_))`.
    #[cfg(feature = "unstable-pre-spec")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
    #[serde(rename = "m.new_content", skip_serializing_if = "Option::is_none")]
    pub new_content: Option<Box<MessageEventContent>>,
}

impl MessageEventContent {
    /// Create a `MessageEventContent` with the given `MessageType`.
    pub fn new(msgtype: MessageType) -> Self {
        Self {
            msgtype,
            relates_to: None,
            #[cfg(feature = "unstable-pre-spec")]
            new_content: None,
        }
    }

    /// A constructor to create a plain text message.
    pub fn text_plain(body: impl Into<String>) -> Self {
        Self::new(MessageType::Text(TextMessageEventContent::plain(body)))
    }

    /// A constructor to create an html message.
    pub fn text_html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self::new(MessageType::Text(TextMessageEventContent::html(body, html_body)))
    }

    /// A constructor to create a plain text notice.
    pub fn notice_plain(body: impl Into<String>) -> Self {
        Self::new(MessageType::Notice(NoticeMessageEventContent::plain(body)))
    }

    /// A constructor to create an html notice.
    pub fn notice_html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self::new(MessageType::Notice(NoticeMessageEventContent::html(body, html_body)))
    }

    /// Creates a plain text reply to a message.
    pub fn text_reply_plain(reply: impl Into<String>, original_message: &MessageEvent) -> Self {
        let quoted = get_plain_quote_fallback(original_message);

        let body = format!("{}\n\n{}", quoted, reply.into());

        Self {
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo { event_id: original_message.event_id.clone() },
            }),
            ..Self::text_plain(body)
        }
    }

    /// Creates a html text reply to a message.
    pub fn text_reply_html(
        reply: impl Into<String>,
        html_reply: impl Into<String>,
        original_message: &MessageEvent,
    ) -> Self {
        let quoted = get_plain_quote_fallback(original_message);
        let quoted_html = get_html_quote_fallback(original_message);

        let body = format!("{}\n\n{}", quoted, reply.into());
        let html_body = format!("{}\n\n{}", quoted_html, html_reply.into());

        Self {
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo { event_id: original_message.event_id.clone() },
            }),
            ..Self::text_html(body, html_body)
        }
    }

    /// Creates a plain text notice reply to a message.
    pub fn notice_reply_plain(reply: impl Into<String>, original_message: &MessageEvent) -> Self {
        let quoted = get_plain_quote_fallback(original_message);

        let body = format!("{}\n\n{}", quoted, reply.into());
        Self {
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo { event_id: original_message.event_id.clone() },
            }),
            ..Self::notice_plain(body)
        }
    }

    /// Creates a html text notice reply to a message.
    pub fn notice_reply_html(
        reply: impl Into<String>,
        html_reply: impl Into<String>,
        original_message: &MessageEvent,
    ) -> Self {
        let quoted = get_plain_quote_fallback(original_message);
        let quoted_html = get_html_quote_fallback(original_message);

        let body = format!("{}\n\n{}", quoted, reply.into());
        let html_body = format!("{}\n\n{}", quoted_html, html_reply.into());

        Self {
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo { event_id: original_message.event_id.clone() },
            }),
            ..Self::notice_html(body, html_body)
        }
    }
}

/// The content that is specific to each message type variant.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
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
    #[cfg(feature = "unstable-pre-spec")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
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
    pub fn new(msgtype: &str, data: JsonObject) -> serde_json::Result<Self> {
        fn from_json_object<T: DeserializeOwned>(obj: JsonObject) -> serde_json::Result<T> {
            serde_json::from_value(JsonValue::Object(obj))
        }

        Ok(match msgtype {
            "m.audio" => Self::Audio(from_json_object(data)?),
            "m.emote" => Self::Emote(from_json_object(data)?),
            "m.file" => Self::File(from_json_object(data)?),
            "m.image" => Self::Image(from_json_object(data)?),
            "m.location" => Self::Location(from_json_object(data)?),
            "m.notice" => Self::Notice(from_json_object(data)?),
            "m.server_notice" => Self::ServerNotice(from_json_object(data)?),
            "m.text" => Self::Text(from_json_object(data)?),
            "m.video" => Self::Video(from_json_object(data)?),
            #[cfg(feature = "unstable-pre-spec")]
            "m.key.verification.request" => Self::VerificationRequest(from_json_object(data)?),
            _ => Self::_Custom(CustomEventContent { msgtype: msgtype.to_owned(), data }),
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
            #[cfg(feature = "unstable-pre-spec")]
            Self::VerificationRequest(_) => "m.key.verification.request",
            Self::_Custom(c) => &c.msgtype,
        }
    }

    /// Returns the associated data.
    ///
    /// Prefer to use the public variants of `MessageType` where possible; this method is meant to
    /// be used for unsupported message types only.
    pub fn data(&self) -> Cow<'_, JsonObject> {
        fn serialize<T: Serialize>(obj: &T) -> JsonObject {
            match serde_json::to_value(obj).expect("message type serialization to succeed") {
                JsonValue::Object(obj) => obj,
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
            #[cfg(feature = "unstable-pre-spec")]
            Self::VerificationRequest(d) => Cow::Owned(serialize(d)),
            Self::_Custom(c) => Cow::Borrowed(&c.data),
        }
    }
}

impl From<MessageType> for MessageEventContent {
    fn from(msgtype: MessageType) -> Self {
        Self::new(msgtype)
    }
}

/// Enum modeling the different ways relationships can be expressed in a
/// `m.relates_to` field of an `m.room.message` event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(from = "RelatesToJsonRepr", into = "RelatesToJsonRepr")]
pub enum Relation {
    /// A reference to another event.
    #[cfg(feature = "unstable-pre-spec")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
    Reference(Reference),

    /// An annotation to an event.
    #[cfg(feature = "unstable-pre-spec")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
    Annotation(Annotation),

    /// An event that replaces another event.
    #[cfg(feature = "unstable-pre-spec")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
    Replacement(Replacement),

    /// An `m.in_reply_to` relation indicating that the event is a reply to
    /// another event.
    Reply {
        /// Information about another message being replied to.
        in_reply_to: InReplyTo,
    },

    /// Custom, unsupported relation.
    #[doc(hidden)]
    _Custom(JsonValue),
}

impl From<Relation> for RelatesToJsonRepr {
    fn from(value: Relation) -> Self {
        match value {
            #[cfg(feature = "unstable-pre-spec")]
            Relation::Annotation(r) => RelatesToJsonRepr::Relation(RelationJsonRepr::Annotation(r)),
            #[cfg(feature = "unstable-pre-spec")]
            Relation::Reference(r) => RelatesToJsonRepr::Relation(RelationJsonRepr::Reference(r)),
            #[cfg(feature = "unstable-pre-spec")]
            Relation::Replacement(r) => {
                RelatesToJsonRepr::Relation(RelationJsonRepr::Replacement(r))
            }
            Relation::Reply { in_reply_to } => RelatesToJsonRepr::Reply { in_reply_to },
            Relation::_Custom(c) => RelatesToJsonRepr::Custom(c),
        }
    }
}

impl From<RelatesToJsonRepr> for Relation {
    fn from(value: RelatesToJsonRepr) -> Self {
        match value {
            #[cfg(feature = "unstable-pre-spec")]
            RelatesToJsonRepr::Relation(r) => match r {
                RelationJsonRepr::Annotation(a) => Self::Annotation(a),
                RelationJsonRepr::Reference(r) => Self::Reference(r),
                RelationJsonRepr::Replacement(r) => Self::Replacement(r),
            },
            RelatesToJsonRepr::Reply { in_reply_to } => Self::Reply { in_reply_to },
            RelatesToJsonRepr::Custom(v) => Self::_Custom(v),
        }
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
    pub url: Option<MxcUri>,

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
    /// Creates a new non-encrypted `AudioMessageEventContent` with the given body, url and optional
    /// extra info.
    pub fn plain(body: String, url: MxcUri, info: Option<Box<AudioInfo>>) -> Self {
        Self { body, url: Some(url), info, file: None }
    }

    /// Creates a new encrypted `AudioMessageEventContent` with the given body and encrypted file.
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
    #[cfg_attr(docsrs, doc(cfg(feature = "markdown")))]
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
    pub url: Option<MxcUri>,

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
    /// Creates a new non-encrypted `FileMessageEventContent` with the given body, url and optional
    /// extra info.
    pub fn plain(body: String, url: MxcUri, info: Option<Box<FileInfo>>) -> Self {
        Self { body, filename: None, url: Some(url), info, file: None }
    }

    /// Creates a new encrypted `FileMessageEventContent` with the given body and encrypted file.
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
    pub thumbnail_url: Option<MxcUri>,

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
    /// This could be the alt text of the image, the filename of the image, or some kind of content
    /// description for accessibility e.g. "image attachment".
    pub body: String,

    /// The URL to the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<MxcUri>,

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
    /// Creates a new non-encrypted `ImageMessageEventContent` with the given body, url and optional
    /// extra info.
    pub fn plain(body: String, url: MxcUri, info: Option<Box<ImageInfo>>) -> Self {
        Self { body, url: Some(url), info, file: None }
    }

    /// Creates a new encrypted `ImageMessageEventContent` with the given body and encrypted file.
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
    /// Creates a new `LocationMessageEventContent` with the given body and geo URI.
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
    pub thumbnail_url: Option<MxcUri>,

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
    #[cfg_attr(docsrs, doc(cfg(feature = "markdown")))]
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
    /// Creates a new `ServerNoticeMessageEventContent` with the given body and notice type.
    pub fn new(body: String, server_notice_type: ServerNoticeType) -> Self {
        Self { body, server_notice_type, admin_contact: None, limit_type: None }
    }
}

/// Types of server notices.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
pub enum ServerNoticeType {
    /// The server has exceeded some limit which requires the server administrator to intervene.
    #[ruma_enum(rename = "m.server_notice.usage_limit_reached")]
    UsageLimitReached,

    #[doc(hidden)]
    _Custom(String),
}

/// Types of usage limits.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
pub enum LimitType {
    /// The server's number of active users in the last 30 days has exceeded the maximum.
    ///
    /// New connections are being refused by the server. What defines "active" is left as an
    /// implementation detail, however servers are encouraged to treat syncing users as "active".
    MonthlyActiveUser,

    #[doc(hidden)]
    _Custom(String),
}

/// The format for the formatted representation of a message body.
///
/// This type can hold an arbitrary string. To check for formats that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
pub enum MessageFormat {
    /// HTML.
    #[ruma_enum(rename = "org.matrix.custom.html")]
    Html,

    #[doc(hidden)]
    _Custom(String),
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
    #[cfg_attr(docsrs, doc(cfg(feature = "markdown")))]
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
    #[cfg_attr(docsrs, doc(cfg(feature = "markdown")))]
    pub fn markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self { formatted: FormattedBody::markdown(&body), ..Self::plain(body) }
    }

    /// A convenience constructor to create a plain text message.
    #[deprecated = "Renamed to plain"]
    pub fn new_plain(body: impl Into<String>) -> Self {
        Self::plain(body)
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
    pub url: Option<MxcUri>,

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
    /// Creates a new non-encrypted `VideoMessageEventContent` with the given body, url and optional
    /// extra info.
    pub fn plain(body: String, url: MxcUri, info: Option<Box<VideoInfo>>) -> Self {
        Self { body, url: Some(url), info, file: None }
    }

    /// Creates a new encrypted `VideoMessageEventContent` with the given body and encrypted file.
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
    #[serde(rename = "h")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<UInt>,

    /// The width of the video in pixels.
    #[serde(rename = "w")]
    #[serde(skip_serializing_if = "Option::is_none")]
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
    pub thumbnail_url: Option<MxcUri>,

    /// Information on the encrypted thumbnail file.
    ///
    /// Only present if the thumbnail is encrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_file: Option<Box<EncryptedFile>>,

    /// The [BlurHash](https://blurha.sh) for this video.
    ///
    /// This uses the unstable prefix in
    /// [MSC2448](https://github.com/matrix-org/matrix-doc/pull/2448).
    #[cfg(feature = "unstable-pre-spec")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
    #[serde(rename = "xyz.amorgan.blurhash")]
    #[serde(skip_serializing_if = "Option::is_none")]
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
#[cfg(feature = "unstable-pre-spec")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.key.verification.request")]
pub struct KeyVerificationRequestEventContent {
    /// A fallback message to alert users that their client does not support the key verification
    /// framework.
    pub body: String,

    /// The verification methods supported by the sender.
    pub methods: Vec<VerificationMethod>,

    /// The device ID which is initiating the request.
    pub from_device: DeviceIdBox,

    /// The user ID which should receive the request.
    ///
    /// Users should only respond to verification requests if they are named in this field. Users
    /// who are not named in this field and who did not send this event should ignore all other
    /// events that have a `m.reference` relationship with this event.
    pub to: UserId,
}

#[cfg(feature = "unstable-pre-spec")]
impl KeyVerificationRequestEventContent {
    /// Creates a new `KeyVerificationRequestEventContent` with the given body, method, device and
    /// user ID.
    pub fn new(
        body: String,
        methods: Vec<VerificationMethod>,
        from_device: DeviceIdBox,
        to: UserId,
    ) -> Self {
        Self { body, methods, from_device, to }
    }
}

/// The payload for a custom message event.
#[doc(hidden)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CustomEventContent {
    /// A custom msgtype
    msgtype: String,

    /// Remaining event content
    #[serde(flatten)]
    data: JsonObject,
}

fn get_plain_quote_fallback(original_message: &MessageEvent) -> String {
    match &original_message.content.msgtype {
        MessageType::Audio(_) => {
            format!("> <{:?}> sent an audio file.", original_message.sender)
        }
        MessageType::Emote(content) => {
            format!("> * <{:?}> {}", original_message.sender, content.body)
        }
        MessageType::File(_) => {
            format!("> <{:?}> sent a file.", original_message.sender)
        }
        MessageType::Image(_) => {
            format!("> <{:?}> sent an image.", original_message.sender)
        }
        MessageType::Location(content) => {
            format!("> <{:?}> {}", original_message.sender, content.body)
        }
        MessageType::Notice(content) => {
            format!("> <{:?}> {}", original_message.sender, content.body)
        }
        MessageType::ServerNotice(content) => {
            format!("> <{:?}> {}", original_message.sender, content.body)
        }
        MessageType::Text(content) => {
            format!("> <{:?}> {}", original_message.sender, content.body)
        }
        MessageType::Video(_) => {
            format!("> <{:?}> sent a video.", original_message.sender)
        }
        MessageType::_Custom(content) => {
            format!(
                "> <{:?}> {}",
                original_message.sender,
                content.data["body"].as_str().unwrap_or(""),
            )
        }
        #[cfg(feature = "unstable-pre-spec")]
        MessageType::VerificationRequest(content) => {
            format!("> <{:?}> {}", original_message.sender, content.body)
        }
    }
}

fn get_html_quote_fallback(original_message: &MessageEvent) -> String {
    match &original_message.content.msgtype {
        MessageType::Audio(_) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a>
                        <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        sent an audio file.
                    </blockquote>
                </mx-reply>
                ",
                room_id = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
            )
        }
        MessageType::Emote(content) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a>
                        * <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        {body}
                    </blockquote>
                </mx-reply>
                ",
                room_id = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
                body = formatted_or_plain_body(&content.formatted, &content.body),
            )
        }
        MessageType::File(_) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a>
                        <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        sent a file.
                    </blockquote>
                </mx-reply>
                ",
                room_id = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
            )
        }
        MessageType::Image(_) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a>
                        <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        sent an image.
                    </blockquote>
                </mx-reply>
                ",
                room_id = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
            )
        }
        MessageType::Location(_) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a>
                        <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        sent a location.
                    </blockquote>
                </mx-reply>
                ",
                room_id = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
            )
        }
        MessageType::Notice(content) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a>
                        <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        {body}
                    </blockquote>
                </mx-reply>
                ",
                room_id = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
                body = formatted_or_plain_body(&content.formatted, &content.body),
            )
        }
        MessageType::ServerNotice(content) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a>
                        <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        {body}
                    </blockquote>
                </mx-reply>
                ",
                room_id = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
                body = content.body,
            )
        }
        MessageType::Text(content) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a>
                        <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        {body}
                    </blockquote>
                </mx-reply>
                ",
                room_id = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
                body = formatted_or_plain_body(&content.formatted, &content.body),
            )
        }
        MessageType::Video(_) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a>
                        <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        sent a video.
                    </blockquote>
                </mx-reply>
                ",
                room_id = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
            )
        }
        MessageType::_Custom(content) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{room_id}/{event_id}\">In reply to</a>
                        <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        {body}
                    </blockquote>
                </mx-reply>
                ",
                room_id = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
                body = content.data["body"].as_str().unwrap_or(""),
            )
        }
        #[cfg(feature = "unstable-pre-spec")]
        MessageType::VerificationRequest(content) => {
            formatdoc!(
                "
                <mx-reply>
                    <blockquote>
                        <a href=\"https://matrix.to/#/{server_name}/{event_id}\">In reply to</a>
                        <a href=\"https://matrix.to/#/{sender}\">{sender}</a>
                        <br />
                        {body}
                    </blockquote>
                </mx-reply>
                ",
                server_name = original_message.room_id,
                event_id = original_message.event_id,
                sender = original_message.sender,
                body = content.body,
            )
        }
    }
}

fn formatted_or_plain_body<'a>(formatted: &'a Option<FormattedBody>, body: &'a str) -> &'a str {
    if let Some(formatted_body) = formatted {
        &formatted_body.body
    } else {
        body
    }
}
