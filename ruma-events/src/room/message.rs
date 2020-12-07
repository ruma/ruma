//! Types for the *m.room.message* event.

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

pub mod feedback;

use crate::MessageEvent as OuterMessageEvent;

/// This event is used when sending messages in a room.
///
/// Messages are not limited to be text.
pub type MessageEvent = OuterMessageEvent<MessageEventContent>;

/// The payload for `MessageEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, MessageEventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.message")]
#[serde(tag = "msgtype")]
pub enum MessageEventContent {
    /// An audio message.
    #[serde(rename = "m.audio")]
    Audio(AudioMessageEventContent),

    /// An emote message.
    #[serde(rename = "m.emote")]
    Emote(EmoteMessageEventContent),

    /// A file message.
    #[serde(rename = "m.file")]
    File(FileMessageEventContent),

    /// An image message.
    #[serde(rename = "m.image")]
    Image(ImageMessageEventContent),

    /// A location message.
    #[serde(rename = "m.location")]
    Location(LocationMessageEventContent),

    /// A notice message.
    #[serde(rename = "m.notice")]
    Notice(NoticeMessageEventContent),

    /// A server notice message.
    #[serde(rename = "m.server_notice")]
    ServerNotice(ServerNoticeMessageEventContent),

    /// A text message.
    #[serde(rename = "m.text")]
    Text(TextMessageEventContent),

    /// A video message.
    #[serde(rename = "m.video")]
    Video(VideoMessageEventContent),

    /// A request to initiate a key verification.
    #[cfg(feature = "unstable-pre-spec")]
    #[serde(rename = "m.key.verification.request")]
    VerificationRequest(KeyVerificationRequestEventContent),
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
pub struct EmoteMessageEventContent {
    /// The emote action to perform.
    pub body: String,

    /// Formatted form of the message `body`.
    #[serde(flatten)]
    pub formatted: Option<FormattedBody>,
}

/// The payload for a file message.
#[derive(Clone, Debug, Deserialize, Serialize)]
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

    /// A convenience constructor to create a plain text message.
    #[deprecated = "Renamed to plain"]
    pub fn new_plain(body: impl Into<String>) -> Self {
        Self::plain(body)
    }
}

/// The payload for a video message.
#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use matches::assert_matches;
    #[cfg(feature = "unstable-pre-spec")]
    use ruma_identifiers::DeviceIdBox;
    use ruma_identifiers::{event_id, room_id, user_id};
    use ruma_serde::Raw;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    #[cfg(feature = "unstable-pre-spec")]
    use super::KeyVerificationRequestEventContent;
    use super::{
        AudioMessageEventContent, FormattedBody, MessageEventContent, MessageFormat, Relation,
    };
    #[cfg(feature = "unstable-pre-spec")]
    use crate::key::verification::VerificationMethod;
    use crate::{
        room::{message::TextMessageEventContent, relationships::InReplyTo},
        MessageEvent, Unsigned,
    };

    #[test]
    fn serialization() {
        let ev = MessageEvent {
            content: MessageEventContent::Audio(AudioMessageEventContent {
                body: "test".into(),
                info: None,
                url: Some("http://example.com/audio.mp3".into()),
                file: None,
            }),
            event_id: event_id!("$143273582443PhrSn:example.org"),
            origin_server_ts: UNIX_EPOCH + Duration::from_millis(10_000),
            room_id: room_id!("!testroomid:example.org"),
            sender: user_id!("@user:example.org"),
            unsigned: Unsigned::default(),
        };

        assert_eq!(
            to_json_value(ev).unwrap(),
            json!({
                "type": "m.room.message",
                "event_id": "$143273582443PhrSn:example.org",
                "origin_server_ts": 10_000,
                "room_id": "!testroomid:example.org",
                "sender": "@user:example.org",
                "content": {
                    "body": "test",
                    "msgtype": "m.audio",
                    "url": "http://example.com/audio.mp3",
                }
            })
        );
    }

    #[test]
    fn content_serialization() {
        let message_event_content = MessageEventContent::Audio(AudioMessageEventContent {
            body: "test".into(),
            info: None,
            url: Some("http://example.com/audio.mp3".into()),
            file: None,
        });

        assert_eq!(
            to_json_value(&message_event_content).unwrap(),
            json!({
                "body": "test",
                "msgtype": "m.audio",
                "url": "http://example.com/audio.mp3"
            })
        );
    }

    #[test]
    fn formatted_body_serialization() {
        let message_event_content = MessageEventContent::Text(TextMessageEventContent {
            body: "Hello, World!".into(),
            formatted: Some(FormattedBody {
                format: MessageFormat::Html,
                body: "Hello, <em>World</em>!".into(),
            }),
            relates_to: None,
            #[cfg(feature = "unstable-pre-spec")]
            new_content: None,
        });

        assert_eq!(
            to_json_value(&message_event_content).unwrap(),
            json!({
                "body": "Hello, World!",
                "msgtype": "m.text",
                "format": "org.matrix.custom.html",
                "formatted_body": "Hello, <em>World</em>!",
            })
        );
    }

    #[test]
    fn plain_text_content_serialization() {
        let message_event_content = MessageEventContent::Text(TextMessageEventContent::plain(
            "> <@test:example.com> test\n\ntest reply",
        ));

        assert_eq!(
            to_json_value(&message_event_content).unwrap(),
            json!({
                "body": "> <@test:example.com> test\n\ntest reply",
                "msgtype": "m.text"
            })
        );
    }

    #[test]
    fn relates_to_content_serialization() {
        let message_event_content = MessageEventContent::Text(TextMessageEventContent {
            body: "> <@test:example.com> test\n\ntest reply".to_owned(),
            formatted: None,
            relates_to: Some(Relation::Reply {
                in_reply_to: InReplyTo { event_id: event_id!("$15827405538098VGFWH:example.com") },
            }),
            #[cfg(feature = "unstable-pre-spec")]
            new_content: None,
        });

        let json_data = json!({
            "body": "> <@test:example.com> test\n\ntest reply",
            "msgtype": "m.text",
            "m.relates_to": {
                "m.in_reply_to": {
                    "event_id": "$15827405538098VGFWH:example.com"
                }
            }
        });

        assert_eq!(to_json_value(&message_event_content).unwrap(), json_data);
    }

    #[test]
    #[cfg(not(feature = "unstable-pre-spec"))]
    fn edit_deserialization_061() {
        let json_data = json!({
            "body": "s/foo/bar",
            "msgtype": "m.text",
            "m.relates_to": {
                "rel_type": "m.replace",
                "event_id": event_id!("$1598361704261elfgc:localhost"),
            },
            "m.new_content": {
                "body": "bar",
            },
        });

        assert_matches!(
            from_json_value::<MessageEventContent>(json_data).unwrap(),
            MessageEventContent::Text(TextMessageEventContent {
                body,
                formatted: None,
                relates_to: Some(Relation::Custom(_)),
            }) if body == "s/foo/bar"
        );
    }

    #[test]
    #[cfg(feature = "unstable-pre-spec")]
    fn edit_deserialization_future() {
        use crate::room::relationships::Replacement;

        let ev_id = event_id!("$1598361704261elfgc:localhost");
        let json_data = json!({
            "body": "s/foo/bar",
            "msgtype": "m.text",
            "m.relates_to": {
                "rel_type": "m.replace",
                "event_id": ev_id,
            },
            "m.new_content": {
                "body": "bar",
                "msgtype": "m.text",
            },
        });

        assert_matches!(
            from_json_value::<MessageEventContent>(json_data).unwrap(),
            MessageEventContent::Text(TextMessageEventContent {
                body,
                formatted: None,
                relates_to: Some(Relation::Replacement(Replacement { event_id })),
                new_content: Some(new_content),
            }) if body == "s/foo/bar"
                && event_id == ev_id
                && matches!(
                    &*new_content,
                    MessageEventContent::Text(TextMessageEventContent {
                        body,
                        formatted: None,
                        relates_to: None,
                        new_content: None,
                    }) if body == "bar"
                )
        );
    }

    #[test]
    #[cfg(feature = "unstable-pre-spec")]
    fn verification_request_deserialization() {
        let user_id = user_id!("@example2:localhost");
        let device_id: DeviceIdBox = "XOWLHHFSWM".into();

        let json_data = json!({
            "body": "@example:localhost is requesting to verify your key, ...",
            "msgtype": "m.key.verification.request",
            "to": user_id,
            "from_device": device_id,
            "methods": [
                "m.sas.v1",
                "m.qr_code.show.v1",
                "m.reciprocate.v1"
            ]
        });

        assert_matches!(
            from_json_value::<MessageEventContent>(json_data).unwrap(),
            MessageEventContent::VerificationRequest(KeyVerificationRequestEventContent {
                body,
                to,
                from_device,
                methods
            }) if body == "@example:localhost is requesting to verify your key, ..."
                && to == user_id
                && from_device == device_id
                && methods.contains(&VerificationMethod::MSasV1)
        );
    }

    #[test]
    #[cfg(feature = "unstable-pre-spec")]
    fn verification_request_serialization() {
        let user_id = user_id!("@example2:localhost");
        let device_id: DeviceIdBox = "XOWLHHFSWM".into();
        let body = "@example:localhost is requesting to verify your key, ...".to_string();

        let methods = vec![
            VerificationMethod::MSasV1,
            VerificationMethod::_Custom("m.qr_code.show.v1".to_string()),
            VerificationMethod::_Custom("m.reciprocate.v1".to_string()),
        ];

        let json_data = json!({
            "body": body,
            "msgtype": "m.key.verification.request",
            "to": user_id,
            "from_device": device_id,
            "methods": methods
        });

        let content =
            MessageEventContent::VerificationRequest(KeyVerificationRequestEventContent {
                to: user_id,
                from_device: device_id,
                body,
                methods,
            });

        assert_eq!(to_json_value(&content).unwrap(), json_data,);
    }

    #[test]
    fn content_deserialization() {
        let json_data = json!({
            "body": "test",
            "msgtype": "m.audio",
            "url": "http://example.com/audio.mp3"
        });

        assert_matches!(
            from_json_value::<Raw<MessageEventContent>>(json_data)
                .unwrap()
                .deserialize()
                .unwrap(),
            MessageEventContent::Audio(AudioMessageEventContent {
                body,
                info: None,
                url: Some(url),
                file: None,
            }) if body == "test" && url == "http://example.com/audio.mp3"
        );
    }

    #[test]
    fn content_deserialization_failure() {
        let json_data = json!({
            "body": "test","msgtype": "m.location",
            "url": "http://example.com/audio.mp3"
        });
        assert!(from_json_value::<Raw<MessageEventContent>>(json_data)
            .unwrap()
            .deserialize()
            .is_err());
    }
}
