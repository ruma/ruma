//! Types for the *m.room.message* event.

use std::collections::BTreeMap;

use js_int::UInt;
use ruma_events_macros::MessageEventContent;
#[cfg(feature = "unstable-pre-spec")]
use ruma_identifiers::{DeviceIdBox, UserId};
use ruma_serde::StringEnum;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[cfg(feature = "unstable-pre-spec")]
use crate::key::verification::VerificationMethod;

#[cfg(feature = "unstable-pre-spec")]
use super::relationships::{Annotation, Reference, RelationJsonRepr, Replacement};
use super::{relationships::RelatesToJsonRepr, EncryptedFile, ImageInfo, ThumbnailInfo};

// FIXME: Do we want to keep re-exporting this?
pub use super::relationships::InReplyTo;

#[doc(hidden)]
pub mod content_serde;
pub mod feedback;

use crate::MessageEvent as OuterMessageEvent;

/// This event is used when sending messages in a room.
///
/// Messages are not limited to be text.
pub type MessageEvent = OuterMessageEvent<MessageEventContent>;

/// The payload for `MessageEvent`.
#[derive(Clone, Debug, Serialize, MessageEventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.message")]
#[serde(untagged)]
pub enum MessageEventContent {
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
    VerificationRequest(KeyVerificationRequestEventContent),

    /// A custom message.
    #[doc(hidden)]
    _Custom(CustomEventContent),
}

/// Enum modeling the different ways relationships can be expressed in a
/// `m.relates_to` field of an m.room.message event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(from = "RelatesToJsonRepr", into = "RelatesToJsonRepr")]
pub enum Relation {
    /// A reference to another event.
    #[cfg(feature = "unstable-pre-spec")]
    Reference(Reference),

    /// An annotation to an event.
    #[cfg(feature = "unstable-pre-spec")]
    Annotation(Annotation),

    /// An event that replaces another event.
    #[cfg(feature = "unstable-pre-spec")]
    Replacement(Replacement),

    /// An `m.in_reply_to` relation indicating that the event is a reply to
    /// another event.
    Reply {
        /// Information about another message being replied to.
        in_reply_to: InReplyTo,
    },

    /// Custom, unsupported relation.
    Custom(JsonValue),
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
            Relation::Custom(c) => RelatesToJsonRepr::Custom(c),
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
            RelatesToJsonRepr::Custom(v) => Self::Custom(v),
        }
    }
}

impl MessageEventContent {
    /// A convenience constructor to create a plain text message.
    pub fn text_plain(body: impl Into<String>) -> Self {
        Self::Text(TextMessageEventContent::plain(body))
    }

    /// A convenience constructor to create an html message.
    pub fn text_html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self::Text(TextMessageEventContent::html(body, html_body))
    }

    /// A convenience constructor to create an plain text notice.
    pub fn notice_plain(body: impl Into<String>) -> Self {
        Self::Notice(NoticeMessageEventContent::plain(body))
    }

    /// A convenience constructor to create an html notice.
    pub fn notice_html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self::Notice(NoticeMessageEventContent::html(body, html_body))
    }
}

/// The payload for an audio message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "msgtype", rename = "m.audio")]
pub struct AudioMessageEventContent {
    /// The textual representation of this message.
    pub body: String,

    /// Metadata for the audio clip referred to in `url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<AudioInfo>>,

    /// The URL to the audio clip. Required if the file is unencrypted. The URL (typically
    /// [MXC URI](https://matrix.org/docs/spec/client_server/r0.6.1#mxc-uri)) to the audio clip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Required if the audio clip is encrypted. Information on the encrypted audio clip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<Box<EncryptedFile>>,
}

/// Metadata about an audio clip.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AudioInfo {
    /// The duration of the audio in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<UInt>,

    /// The mimetype of the audio, e.g. "audio/aac."
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// The size of the audio clip in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<UInt>,
}

/// The payload for an emote message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "msgtype", rename = "m.emote")]
pub struct EmoteMessageEventContent {
    /// The emote action to perform.
    pub body: String,

    /// Formatted form of the message `body`.
    #[serde(flatten)]
    pub formatted: Option<FormattedBody>,
}

/// The payload for a file message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "msgtype", rename = "m.file")]
pub struct FileMessageEventContent {
    /// A human-readable description of the file. This is recommended to be the filename of the
    /// original upload.
    pub body: String,

    /// The original filename of the uploaded file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,

    /// Metadata about the file referred to in `url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<FileInfo>>,

    /// The URL to the file. Required if the file is unencrypted. The URL (typically
    /// [MXC URI](https://matrix.org/docs/spec/client_server/r0.6.1#mxc-uri)) to the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Required if file is encrypted. Information on the encrypted file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<Box<EncryptedFile>>,
}

/// Metadata about a file.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileInfo {
    /// The mimetype of the file, e.g. "application/msword."
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// The size of the file in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<UInt>,

    /// Metadata about the image referred to in `thumbnail_url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_info: Option<Box<ThumbnailInfo>>,

    /// The URL to the thumbnail of the file. Only present if the thumbnail is unencrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,

    /// Information on the encrypted thumbnail file. Only present if the thumbnail is encrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_file: Option<Box<EncryptedFile>>,
}

/// The payload for an image message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "msgtype", rename = "m.image")]
pub struct ImageMessageEventContent {
    /// A textual representation of the image. This could be the alt text of the image, the
    /// filename of the image, or some kind of content description for accessibility e.g.
    /// "image attachment."
    pub body: String,

    /// Metadata about the image referred to in `url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<ImageInfo>>,

    /// The URL to the image. Required if the file is unencrypted. The URL (typically
    /// [MXC URI](https://matrix.org/docs/spec/client_server/r0.6.1#mxc-uri)) to the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Required if image is encrypted. Information on the encrypted image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<Box<EncryptedFile>>,
}

/// The payload for a location message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "msgtype", rename = "m.location")]
pub struct LocationMessageEventContent {
    /// A description of the location e.g. "Big Ben, London, UK,"or some kind of content
    /// description for accessibility, e.g. "location attachment."
    pub body: String,

    /// A geo URI representing the location.
    pub geo_uri: String,

    /// Info about the location being represented.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<LocationInfo>>,
}

/// Thumbnail info associated with a location.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LocationInfo {
    /// Metadata about the image referred to in `thumbnail_url` or `thumbnail_file`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_info: Option<Box<ThumbnailInfo>>,

    /// The URL to a thumbnail of the location being represented. Only present if the thumbnail is
    /// unencrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,

    /// Information on an encrypted thumbnail of the location being represented. Only present if
    /// the thumbnail is encrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_file: Option<Box<EncryptedFile>>,
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

    /// Information about related messages for
    /// [rich replies](https://matrix.org/docs/spec/client_server/r0.6.1#rich-replies).
    #[serde(rename = "m.relates_to", skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Relation>,

    /// New content of an edited message.
    ///
    /// This should only be set if `relates_to` is `Some(Relation::Replacement(_))`.
    #[cfg(feature = "unstable-pre-spec")]
    #[serde(rename = "m.new_content", skip_serializing_if = "Option::is_none")]
    pub new_content: Option<Box<MessageEventContent>>,
}

impl NoticeMessageEventContent {
    /// A convenience constructor to create a plain text notice.
    pub fn plain(body: impl Into<String>) -> Self {
        Self {
            body: body.into(),
            formatted: None,
            relates_to: None,
            #[cfg(feature = "unstable-pre-spec")]
            new_content: None,
        }
    }

    /// A convenience constructor to create an html notice.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self { formatted: Some(FormattedBody::html(html_body)), ..Self::plain(body) }
    }
}

/// The payload for a server notice message.
#[derive(Clone, Debug, Deserialize, Serialize)]
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
/// This type can hold an arbitrary string. To check for events that are not
/// available as a documented variant here, use its string representation,
/// obtained through `.as_str()`.
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

    /// Information about related messages for
    /// [rich replies](https://matrix.org/docs/spec/client_server/r0.6.1#rich-replies).
    #[serde(rename = "m.relates_to", skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Relation>,

    /// New content of an edited message.
    ///
    /// This should only be set if `relates_to` is `Some(Relation::Replacement(_))`.
    #[cfg(feature = "unstable-pre-spec")]
    #[serde(rename = "m.new_content", skip_serializing_if = "Option::is_none")]
    pub new_content: Option<Box<MessageEventContent>>,
}

impl TextMessageEventContent {
    /// A convenience constructor to create a plain text message.
    pub fn plain(body: impl Into<String>) -> Self {
        Self {
            body: body.into(),
            formatted: None,
            relates_to: None,
            #[cfg(feature = "unstable-pre-spec")]
            new_content: None,
        }
    }

    /// A convenience constructor to create an html message.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self { formatted: Some(FormattedBody::html(html_body)), ..Self::plain(body) }
    }

    /// A convenience constructor to create a markdown message.
    #[cfg(feature = "markdown")]
    pub fn markdown(body: impl Into<String>) -> Self {
        let body = body.into();
        let mut html_body = String::new();
        pulldown_cmark::html::push_html(&mut html_body, pulldown_cmark::Parser::new(&body));
        Self::html(body, html_body)
    }

    /// A convenience constructor to create a plain text message.
    #[deprecated = "Renamed to plain"]
    pub fn new_plain(body: impl Into<String>) -> Self {
        Self::plain(body)
    }
}

/// The payload for a video message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "msgtype", rename = "m.video")]
pub struct VideoMessageEventContent {
    /// A description of the video, e.g. "Gangnam Style," or some kind of content description for
    /// accessibility, e.g. "video attachment."
    pub body: String,

    /// Metadata about the video clip referred to in `url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<VideoInfo>>,

    /// The URL to the video clip.  Required if the file is unencrypted. The URL (typically
    /// [MXC URI](https://matrix.org/docs/spec/client_server/r0.6.1#mxc-uri)) to the video clip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Required if video clip is encrypted. Information on the encrypted video clip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<Box<EncryptedFile>>,
}

/// Metadata about a video.
#[derive(Clone, Debug, Deserialize, Serialize)]
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

    /// The mimetype of the video, e.g. "video/mp4."
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// The size of the video in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<UInt>,

    /// Metadata about an image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_info: Option<Box<ThumbnailInfo>>,

    /// The URL (typically [MXC URI](https://matrix.org/docs/spec/client_server/r0.6.1#mxc-uri)) to
    /// an image thumbnail of the video clip. Only present if the thumbnail is unencrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,

    /// Information on the encrypted thumbnail file.  Only present if the thumbnail is encrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_file: Option<Box<EncryptedFile>>,

    /// The [BlurHash](https://blurha.sh) for this video.
    ///
    /// This uses the unstable prefix in
    /// [MSC2448](https://github.com/matrix-org/matrix-doc/pull/2448).
    #[cfg(feature = "unstable-pre-spec")]
    #[serde(rename = "xyz.amorgan.blurhash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blurhash: Option<String>,
}

/// The payload for a key verification request message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg(feature = "unstable-pre-spec")]
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
    /// events that have a m.reference relationship with this event.
    pub to: UserId,
}

/// The payload for a custom message event.
#[doc(hidden)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CustomEventContent {
    /// A custom msgtype
    pub msgtype: String,

    /// Remaining event content
    #[serde(flatten)]
    pub data: BTreeMap<String, JsonValue>,
}
