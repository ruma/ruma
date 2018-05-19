//! Types for the *m.room.message* event.

use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{from_value, Value};

use super::{ImageInfo, ThumbnailInfo};

room_event! {
    /// A message sent to a room.
    pub struct MessageEvent(MessageEventContent) {}
}

/// The message type of message event, e.g. `m.image` or `m.text`.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
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

    /// A text message.
    #[serde(rename = "m.text")]
    Text,

    /// A video message.
    #[serde(rename = "m.video")]
    Video,
}

/// The payload of a message event.
#[derive(Clone, Debug, PartialEq)]
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

    /// An text message.
    Text(TextMessageEventContent),

    /// A video message.
    Video(VideoMessageEventContent),
}

/// The payload of an audio message.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AudioMessageEventContent {
    /// The textual representation of this message.
    pub body: String,
    /// Metadata for the audio clip referred to in `url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<AudioInfo>,
    /// The message type. Always *m.audio*.
    pub msgtype: MessageType,
    /// The URL to the audio clip.
    pub url: String,
}

/// Metadata about an audio clip.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AudioInfo {
    /// The duration of the audio in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,
    /// The mimetype of the audio, e.g. "audio/aac."
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,
    /// The size of the audio clip in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// The payload of an emote message.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EmoteMessageEventContent {
    /// The emote action to perform.
    pub body: String,
    /// The message type. Always *m.emote*.
    pub msgtype: MessageType,
}

/// The payload of a file message.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FileMessageEventContent {
    /// A human-readable description of the file. This is recommended to be the filename of the
    /// original upload.
    pub body: String,
    /// The original filename of the uploaded file.
    pub filename: String,
    /// Metadata about the file referred to in `url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<FileInfo>,
    /// The message type. Always *m.file*.
    pub msgtype: MessageType,
    /// The URL to the file.
    pub url: String,
}

/// Metadata about a file.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FileInfo {
    /// The mimetype of the file, e.g. "application/msword."
    pub mimetype: String,
    /// The size of the file in bytes.
    pub size: u64,
    /// Metadata about the image referred to in `thumbnail_url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_info: Option<ThumbnailInfo>,
    /// The URL to the thumbnail of the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
}

/// The payload of an image message.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ImageMessageEventContent {
    /// A textual representation of the image. This could be the alt text of the image, the filename
    /// of the image, or some kind of content description for accessibility e.g. "image attachment."
    pub body: String,
    /// Metadata about the image referred to in `url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<ImageInfo>,
    /// The message type. Always *m.image*.
    pub msgtype: MessageType,
    /// The URL to the image.
    pub url: String,
}

/// The payload of a location message.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct LocationMessageEventContent {
    /// A description of the location e.g. "Big Ben, London, UK,"or some kind of content description
    /// for accessibility, e.g. "location attachment."
    pub body: String,
    /// A geo URI representing the location.
    pub geo_uri: String,
    /// The message type. Always *m.location*.
    pub msgtype: MessageType,
    /// Info about the location being represented.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<LocationInfo>,
}

/// Thumbnail info associated with a location.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct LocationInfo {
    /// Metadata about the image referred to in `thumbnail_url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_info: Option<ThumbnailInfo>,
    /// The URL to a thumbnail of the location being represented.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
}

/// The payload of a notice message.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NoticeMessageEventContent {
    /// The notice text to send.
    pub body: String,
    /// The message type. Always *m.notice*.
    pub msgtype: MessageType,
}

/// The payload of a text message.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TextMessageEventContent {
    /// The body of the message.
    pub body: String,
    /// The message type. Always *m.text*.
    pub msgtype: MessageType,
}

/// The payload of a video message.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VideoMessageEventContent {
    /// A description of the video, e.g. "Gangnam Style," or some kind of content description for
    /// accessibility, e.g. "video attachment."
    pub body: String,
    /// Metadata about the video clip referred to in `url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<VideoInfo>,
    /// The message type. Always *m.video*.
    pub msgtype: MessageType,
    /// The URL to the video clip.
    pub url: String,
}

/// Metadata about a video.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VideoInfo {
    /// The duration of the video in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,
    /// The height of the video in pixels.
    #[serde(rename = "h")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u64>,
    /// The mimetype of the video, e.g. "video/mp4."
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,
    /// The size of the video in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// Metadata about an image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_info: Option<ThumbnailInfo>,
    /// The URL to a thumbnail of the video clip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    /// The width of the video in pixels.
    #[serde(rename = "w")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u64>,
}

impl_enum! {
    MessageType {
        Audio => "m.audio",
        Emote => "m.emote",
        File => "m.file",
        Image => "m.image",
        Location => "m.location",
        Notice => "m.notice",
        Text => "m.text",
        Video => "m.video",
    }
}

impl Serialize for MessageEventContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            MessageEventContent::Audio(ref content) => content.serialize(serializer),
            MessageEventContent::Emote(ref content) => content.serialize(serializer),
            MessageEventContent::File(ref content) => content.serialize(serializer),
            MessageEventContent::Image(ref content) => content.serialize(serializer),
            MessageEventContent::Location(ref content) => content.serialize(serializer),
            MessageEventContent::Notice(ref content) => content.serialize(serializer),
            MessageEventContent::Text(ref content) => content.serialize(serializer),
            MessageEventContent::Video(ref content) => content.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for MessageEventContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Deserialize::deserialize(deserializer)?;

        let message_type_value = match value.get("msgtype") {
            Some(value) => value.clone(),
            None => return Err(D::Error::missing_field("msgtype")),
        };

        let message_type = match from_value::<MessageType>(message_type_value.clone()) {
            Ok(message_type) => message_type,
            Err(error) => return Err(D::Error::custom(error.to_string())),
        };

        match message_type {
            MessageType::Audio => {
                let content = match from_value::<AudioMessageEventContent>(value) {
                    Ok(content) => content,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(MessageEventContent::Audio(content))
            }
            MessageType::Emote => {
                let content = match from_value::<EmoteMessageEventContent>(value) {
                    Ok(content) => content,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(MessageEventContent::Emote(content))
            }
            MessageType::File => {
                let content = match from_value::<FileMessageEventContent>(value) {
                    Ok(content) => content,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(MessageEventContent::File(content))
            }
            MessageType::Image => {
                let content = match from_value::<ImageMessageEventContent>(value) {
                    Ok(content) => content,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(MessageEventContent::Image(content))
            }
            MessageType::Location => {
                let content = match from_value::<LocationMessageEventContent>(value) {
                    Ok(content) => content,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(MessageEventContent::Location(content))
            }
            MessageType::Notice => {
                let content = match from_value::<NoticeMessageEventContent>(value) {
                    Ok(content) => content,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(MessageEventContent::Notice(content))
            }
            MessageType::Text => {
                let content = match from_value::<TextMessageEventContent>(value) {
                    Ok(content) => content,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(MessageEventContent::Text(content))
            }
            MessageType::Video => {
                let content = match from_value::<VideoMessageEventContent>(value) {
                    Ok(content) => content,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(MessageEventContent::Video(content))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_str, to_string};

    use super::{AudioMessageEventContent, MessageEventContent, MessageType};

    #[test]
    fn serialization() {
        let message_event_content = MessageEventContent::Audio(AudioMessageEventContent {
            body: "test".to_string(),
            info: None,
            msgtype: MessageType::Audio,
            url: "http://example.com/audio.mp3".to_string(),
        });

        assert_eq!(
            to_string(&message_event_content).unwrap(),
            r#"{"body":"test","msgtype":"m.audio","url":"http://example.com/audio.mp3"}"#
        );
    }

    #[test]
    fn deserialization() {
        let message_event_content = MessageEventContent::Audio(AudioMessageEventContent {
            body: "test".to_string(),
            info: None,
            msgtype: MessageType::Audio,
            url: "http://example.com/audio.mp3".to_string(),
        });

        assert_eq!(
            from_str::<MessageEventContent>(
                r#"{"body":"test","msgtype":"m.audio","url":"http://example.com/audio.mp3"}"#
            ).unwrap(),
            message_event_content
        );
    }

    #[test]
    fn deserialization_failure() {
        assert!(
            from_str::<MessageEventContent>(
                r#"{"body":"test","msgtype":"m.location","url":"http://example.com/audio.mp3"}"#
            ).is_err()
        );
    }
}
