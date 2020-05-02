//! Types for the *m.room.message* event.

use std::time::SystemTime;

use js_int::UInt;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{Deserialize, Serialize};

use super::{encrypted::EncryptedEventContent, EncryptedFile, ImageInfo, ThumbnailInfo};
use crate::{EventType, FromRaw, UnsignedData};

pub mod feedback;

/// A message sent to a room.
#[derive(Clone, Debug, Serialize)]
#[serde(rename = "m.room.message", tag = "type")]
pub struct MessageEvent {
    /// The event's content.
    pub content: MessageEventContent,

    /// The unique identifier for the event.
    pub event_id: EventId,

    /// Time on originating homeserver when this event was sent.
    #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
    pub origin_server_ts: SystemTime,

    /// The unique identifier for the room associated with this event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<RoomId>,

    /// The unique identifier for the user who sent this event.
    pub sender: UserId,

    /// Additional key-value pairs not signed by the homeserver.
    #[serde(skip_serializing_if = "UnsignedData::is_empty")]
    pub unsigned: UnsignedData,
}

/// The payload for `MessageEvent`.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Serialize)]
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

    /// A encrypted message.
    Encrypted(EncryptedEventContent),
}

impl FromRaw for MessageEvent {
    type Raw = raw::MessageEvent;

    fn from_raw(raw: raw::MessageEvent) -> Self {
        Self {
            content: FromRaw::from_raw(raw.content),
            event_id: raw.event_id,
            origin_server_ts: raw.origin_server_ts,
            room_id: raw.room_id,
            sender: raw.sender,
            unsigned: raw.unsigned,
        }
    }
}

impl FromRaw for MessageEventContent {
    type Raw = raw::MessageEventContent;

    fn from_raw(raw: raw::MessageEventContent) -> Self {
        use raw::MessageEventContent::*;

        match raw {
            Audio(content) => MessageEventContent::Audio(content),
            Emote(content) => MessageEventContent::Emote(content),
            File(content) => MessageEventContent::File(content),
            Image(content) => MessageEventContent::Image(content),
            Location(content) => MessageEventContent::Location(content),
            Notice(content) => MessageEventContent::Notice(content),
            ServerNotice(content) => MessageEventContent::ServerNotice(content),
            Text(content) => MessageEventContent::Text(content),
            Video(content) => MessageEventContent::Video(content),
            Encrypted(content) => MessageEventContent::Encrypted(content),
        }
    }
}

impl_room_event!(MessageEvent, MessageEventContent, EventType::RoomMessage);

pub(crate) mod raw {
    use std::time::SystemTime;

    use ruma_identifiers::{EventId, RoomId, UserId};
    use serde::{de::DeserializeOwned, Deserialize, Deserializer};
    use serde_json::{from_value as from_json_value, Value as JsonValue};

    use super::{
        AudioMessageEventContent, EmoteMessageEventContent, EncryptedEventContent,
        FileMessageEventContent, ImageMessageEventContent, LocationMessageEventContent,
        MessageType, NoticeMessageEventContent, ServerNoticeMessageEventContent,
        TextMessageEventContent, VideoMessageEventContent,
    };
    use crate::UnsignedData;

    /// A message sent to a room.
    #[derive(Clone, Debug, Deserialize)]
    pub struct MessageEvent {
        /// The event's content.
        pub content: MessageEventContent,

        /// The unique identifier for the event.
        pub event_id: EventId,

        /// Time on originating homeserver when this event was sent.
        #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
        pub origin_server_ts: SystemTime,

        /// The unique identifier for the room associated with this event.
        pub room_id: Option<RoomId>,

        /// The unique identifier for the user who sent this event.
        pub sender: UserId,

        /// Additional key-value pairs not signed by the homeserver.
        #[serde(default)]
        pub unsigned: UnsignedData,
    }

    /// The payload for `MessageEvent`.
    #[allow(clippy::large_enum_variant)]
    #[derive(Clone, Debug)]
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

        /// An text message.
        Text(TextMessageEventContent),

        /// A video message.
        Video(VideoMessageEventContent),

        /// A video message.
        Encrypted(EncryptedEventContent),
    }

    impl<'de> Deserialize<'de> for MessageEventContent {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            use serde::de::Error as _;

            fn deserialize_content<T>(
                c: JsonValue,
                v: fn(T) -> MessageEventContent,
            ) -> Result<MessageEventContent, serde_json::Error>
            where
                T: DeserializeOwned,
            {
                from_json_value::<T>(c).map(v)
            }

            let content: JsonValue = Deserialize::deserialize(deserializer)?;

            let message_type_value = match content.get("msgtype") {
                Some(value) => value.clone(),
                None => return Err(D::Error::missing_field("msgtype")),
            };

            let message_type = match from_json_value::<MessageType>(message_type_value) {
                Ok(message_type) => message_type,
                Err(error) => return Err(D::Error::custom(error)),
            };

            match message_type {
                MessageType::Audio => deserialize_content(content, Self::Audio),
                MessageType::Emote => deserialize_content(content, Self::Emote),
                MessageType::File => deserialize_content(content, Self::File),
                MessageType::Image => deserialize_content(content, Self::Image),
                MessageType::Location => deserialize_content(content, Self::Location),
                MessageType::Notice => deserialize_content(content, Self::Notice),
                MessageType::ServerNotice => deserialize_content(content, Self::ServerNotice),
                MessageType::Text => deserialize_content(content, Self::Text),
                MessageType::Video => deserialize_content(content, Self::Video),
            }
            .map_err(D::Error::custom)
        }
    }
}

/// The message type of message event, e.g. `m.image` or `m.text`.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum MessageType {
    /// An audio message.
    #[serde(rename = "m.audio")]
    Audio,

    /// An emote message.
    #[serde(rename = "m.emote")]
    Emote,

    /// A file message.
    #[serde(rename = "m.file")]
    File,

    /// An image message.
    #[serde(rename = "m.image")]
    Image,

    /// A location message.
    #[serde(rename = "m.location")]
    Location,

    /// A notice message.
    #[serde(rename = "m.notice")]
    Notice,

    /// A server notice.
    #[serde(rename = "m.server_notice")]
    ServerNotice,

    /// A text message.
    #[serde(rename = "m.text")]
    Text,

    /// A video message.
    #[serde(rename = "m.video")]
    Video,
}

/// The payload for an audio message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "msgtype", rename = "m.audio")]
pub struct AudioMessageEventContent {
    /// The textual representation of this message.
    pub body: String,

    /// Metadata for the audio clip referred to in `url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<AudioInfo>,

    /// The URL to the audio clip. Required if the file is unencrypted. The URL (typically
    /// [MXC URI](https://matrix.org/docs/spec/client_server/r0.5.0#mxc-uri)) to the audio clip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Required if the audio clip is encrypted. Information on the encrypted audio clip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<EncryptedFile>,
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

    /// The format used in the `formatted_body`. Currently only `org.matrix.custom.html` is
    /// supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// The formatted version of the `body`. This is required if `format` is specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatted_body: Option<String>,
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
    pub info: Option<FileInfo>,

    /// The URL to the file. Required if the file is unencrypted. The URL (typically
    /// [MXC URI](https://matrix.org/docs/spec/client_server/r0.5.0#mxc-uri)) to the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Required if file is encrypted. Information on the encrypted file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<EncryptedFile>,
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
    pub thumbnail_info: Option<ThumbnailInfo>,

    /// The URL to the thumbnail of the file. Only present if the thumbnail is unencrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,

    /// Information on the encrypted thumbnail file. Only present if the thumbnail is encrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_file: Option<EncryptedFile>,
}

/// The payload for an image message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "msgtype", rename = "m.image")]
pub struct ImageMessageEventContent {
    /// A textual representation of the image. This could be the alt text of the image, the filename
    /// of the image, or some kind of content description for accessibility e.g. "image attachment."
    pub body: String,

    /// Metadata about the image referred to in `url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<ImageInfo>,

    /// The URL to the image. Required if the file is unencrypted. The URL (typically
    /// [MXC URI](https://matrix.org/docs/spec/client_server/r0.5.0#mxc-uri)) to the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Required if image is encrypted. Information on the encrypted image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<EncryptedFile>,
}

/// The payload for a location message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "msgtype", rename = "m.location")]
pub struct LocationMessageEventContent {
    /// A description of the location e.g. "Big Ben, London, UK,"or some kind of content description
    /// for accessibility, e.g. "location attachment."
    pub body: String,

    /// A geo URI representing the location.
    pub geo_uri: String,

    /// Info about the location being represented.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<LocationInfo>,
}

/// Thumbnail info associated with a location.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LocationInfo {
    /// Metadata about the image referred to in `thumbnail_url` or `thumbnail_file`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_info: Option<ThumbnailInfo>,

    /// The URL to a thumbnail of the location being represented. Only present if the thumbnail is
    /// unencrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,

    /// Information on an encrypted thumbnail of the location being represented. Only present if the
    /// thumbnail is encrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_file: Option<EncryptedFile>,
}

/// The payload for a notice message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "msgtype", rename = "m.notice")]
pub struct NoticeMessageEventContent {
    /// The notice text to send.
    pub body: String,

    /// The format used in the `formatted_body`. Currently only `org.matrix.custom.html` is
    /// supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// The formatted version of the `body`. This is required if `format` is specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatted_body: Option<String>,

    /// Information about related messages for
    /// [rich replies](https://matrix.org/docs/spec/client_server/r0.5.0#rich-replies).
    #[serde(rename = "m.relates_to", skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<RelatesTo>,
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
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum ServerNoticeType {
    /// The server has exceeded some limit which requires the server administrator to intervene.
    #[serde(rename = "m.server_notice.usage_limit_reached")]
    UsageLimitReached,
}

/// Types of usage limits.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LimitType {
    /// The server's number of active users in the last 30 days has exceeded the maximum.
    ///
    /// New connections are being refused by the server. What defines "active" is left as an
    /// implementation detail, however servers are encouraged to treat syncing users as "active".
    MonthlyActiveUser,
}

/// The payload for a text message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "msgtype", rename = "m.text")]
pub struct TextMessageEventContent {
    /// The body of the message.
    pub body: String,

    /// The format used in the `formatted_body`. Currently only `org.matrix.custom.html` is
    /// supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// The formatted version of the `body`. This is required if `format` is specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatted_body: Option<String>,

    /// Information about related messages for
    /// [rich replies](https://matrix.org/docs/spec/client_server/r0.5.0#rich-replies).
    #[serde(rename = "m.relates_to", skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<RelatesTo>,
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
    pub info: Option<VideoInfo>,

    /// The URL to the video clip.  Required if the file is unencrypted. The URL (typically
    /// [MXC URI](https://matrix.org/docs/spec/client_server/r0.5.0#mxc-uri)) to the video clip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Required if video clip is encrypted. Information on the encrypted video clip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<EncryptedFile>,
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
    pub thumbnail_info: Option<ThumbnailInfo>,

    /// The URL (typically [MXC URI](https://matrix.org/docs/spec/client_server/r0.5.0#mxc-uri)) to
    /// an image thumbnail of the video clip. Only present if the thumbnail is unencrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,

    /// Information on the encrypted thumbnail file.  Only present if the thumbnail is encrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_file: Option<EncryptedFile>,
}

/// Information about related messages for
/// [rich replies](https://matrix.org/docs/spec/client_server/r0.5.0#rich-replies).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RelatesTo {
    /// Information about another message being replied to.
    #[serde(rename = "m.in_reply_to")]
    pub in_reply_to: InReplyTo,
}

/// Information about the event a "rich reply" is replying to.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InReplyTo {
    /// The event being replied to.
    pub event_id: EventId,
}

impl_enum! {
    MessageType {
        Audio => "m.audio",
        Emote => "m.emote",
        File => "m.file",
        Image => "m.image",
        Location => "m.location",
        Notice => "m.notice",
        ServerNotice => "m.server_notice",
        Text => "m.text",
        Video => "m.video",
    }
}

impl TextMessageEventContent {
    /// A convenience constructor to create a plain text message
    pub fn new_plain(body: impl Into<String>) -> TextMessageEventContent {
        TextMessageEventContent {
            body: body.into(),
            format: None,
            formatted_body: None,
            relates_to: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use matches::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{AudioMessageEventContent, MessageEventContent};
    use crate::room::message::{InReplyTo, RelatesTo, TextMessageEventContent};
    use crate::EventJson;
    use ruma_identifiers::EventId;
    use std::convert::TryFrom;

    #[test]
    fn serialization() {
        let message_event_content = MessageEventContent::Audio(AudioMessageEventContent {
            body: "test".to_string(),
            info: None,
            url: Some("http://example.com/audio.mp3".to_string()),
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
    fn plain_text() {
        let message_event_content = MessageEventContent::Text(TextMessageEventContent::new_plain(
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
    fn relates_to_serialization() {
        let message_event_content = MessageEventContent::Text(TextMessageEventContent {
            body: "> <@test:example.com> test\n\ntest reply".to_owned(),
            format: None,
            formatted_body: None,
            relates_to: Some(RelatesTo {
                in_reply_to: InReplyTo {
                    event_id: EventId::try_from("$15827405538098VGFWH:example.com").unwrap(),
                },
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
    fn deserialization() {
        let json_data = json!({
            "body": "test",
            "msgtype": "m.audio",
            "url": "http://example.com/audio.mp3"
        });

        assert_matches!(
            from_json_value::<EventJson<MessageEventContent>>(json_data)
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
    fn deserialization_failure() {
        let json_data = json!({
            "body": "test","msgtype": "m.location",
            "url": "http://example.com/audio.mp3"
        });
        assert!(from_json_value::<EventJson<MessageEventContent>>(json_data)
            .unwrap()
            .deserialize()
            .is_err());
    }
}
