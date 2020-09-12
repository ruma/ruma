//! Types for the *m.room.message* event.

use std::fmt;

use js_int::UInt;
use ruma_events_macros::MessageEventContent;
use ruma_identifiers::EventId;
use serde::{Deserialize, Serialize};

use super::{EncryptedFile, ImageInfo, ThumbnailInfo};

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
    /// A textual representation of the image. This could be the alt text of the image, the filename
    /// of the image, or some kind of content description for accessibility e.g. "image attachment."
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
    /// A description of the location e.g. "Big Ben, London, UK,"or some kind of content description
    /// for accessibility, e.g. "location attachment."
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

    /// Information on an encrypted thumbnail of the location being represented. Only present if the
    /// thumbnail is encrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_file: Option<Box<EncryptedFile>>,
}

/// The payload for a notice message.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NoticeMessageEventContent {
    /// The notice text to send.
    pub body: String,

    /// Formatted form of the message `body`.
    #[serde(flatten)]
    pub formatted: Option<FormattedBody>,

    /// Information about related messages for
    /// [rich replies](https://matrix.org/docs/spec/client_server/r0.6.1#rich-replies).
    #[serde(rename = "m.relates_to", skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<RelatesTo>,
}

impl NoticeMessageEventContent {
    /// A convenience constructor to create a plain text notice.
    pub fn plain(body: impl Into<String>) -> Self {
        Self { body: body.into(), formatted: None, relates_to: None }
    }

    /// A convenience constructor to create an html notice.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self {
            body: body.into(),
            formatted: Some(FormattedBody::html(html_body)),
            relates_to: None,
        }
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
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum ServerNoticeType {
    /// The server has exceeded some limit which requires the server administrator to intervene.
    #[serde(rename = "m.server_notice.usage_limit_reached")]
    UsageLimitReached,
}

/// Types of usage limits.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(rename_all = "snake_case")]
pub enum LimitType {
    /// The server's number of active users in the last 30 days has exceeded the maximum.
    ///
    /// New connections are being refused by the server. What defines "active" is left as an
    /// implementation detail, however servers are encouraged to treat syncing users as "active".
    MonthlyActiveUser,
}

/// The format for the formatted representation of a message body.
///
/// This type can hold an arbitrary string. To check for events that are not
/// available as a documented variant here, use its string representation,
/// obtained through `.as_str()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum MessageFormat {
    /// HTML.
    #[serde(rename = "org.matrix.custom.html")]
    Html,

    #[doc(hidden)]
    _Custom(String),
}

impl MessageFormat {
    /// Creates a string slice from this `MessageFormat`.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Html => "org.matrix.custom.html",
            Self::_Custom(ref message_format) => message_format,
        }
    }
}

impl fmt::Display for MessageFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
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
pub struct TextMessageEventContent {
    /// The body of the message.
    pub body: String,

    /// Formatted form of the message `body`.
    #[serde(flatten)]
    pub formatted: Option<FormattedBody>,

    /// Information about related messages for
    /// [rich replies](https://matrix.org/docs/spec/client_server/r0.6.1#rich-replies).
    #[serde(rename = "m.relates_to", skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<RelatesTo>,
}

impl TextMessageEventContent {
    /// A convenience constructor to create a plain text message.
    pub fn plain(body: impl Into<String>) -> Self {
        Self { body: body.into(), formatted: None, relates_to: None }
    }

    /// A convenience constructor to create an html message.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self {
            body: body.into(),
            formatted: Some(FormattedBody::html(html_body)),
            relates_to: None,
        }
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
}

/// Information about related messages for
/// [rich replies](https://matrix.org/docs/spec/client_server/r0.6.1#rich-replies).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RelatesTo {
    /// Information about another message being replied to.
    #[serde(rename = "m.in_reply_to")]
    pub in_reply_to: Option<InReplyTo>,
}

/// Information about the event a "rich reply" is replying to.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InReplyTo {
    /// The event being replied to.
    pub event_id: EventId,
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use matches::assert_matches;
    use ruma_common::Raw;
    use ruma_identifiers::{event_id, room_id, user_id};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{AudioMessageEventContent, FormattedBody, MessageEventContent, MessageFormat};
    use crate::{
        room::message::{InReplyTo, RelatesTo, TextMessageEventContent},
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
            relates_to: Some(RelatesTo {
                in_reply_to: Some(InReplyTo {
                    event_id: event_id!("$15827405538098VGFWH:example.com"),
                }),
            }),
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
