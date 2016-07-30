//! Types for the *m.room.message* event.

use RoomEvent;
use super::ImageInfo;

/// A message sent to a room.
pub type MessageEvent = RoomEvent<MessageEventContent, ()>;

/// The message type of message event, e.g. `m.image` or `m.text`.
#[derive(Debug, PartialEq)]
pub enum MessageType {
    /// An audio message.
    Audio,

    /// An emote message.
    Emote,

    /// A file message.
    File,

    /// An image message.
    Image,

    /// A location message.
    Location,

    /// A notice message.
    Notice,

    /// A text message.
    Text,

    /// A video message.
    Video,
}

/// The payload of a message event.
#[derive(Debug, Deserialize, Serialize)]
pub enum MessageEventContent {
    /// An audio message.
    Audio(AudioMessageEventContent),

    /// An emote message.
    Emote(EmoteMessageEventContent),

    /// An file message.
    File(FileMessageEventContent),

    /// An image message.
    Image(ImageMessageEventContent),

    /// An location message.
    Location(LocationMessageEventContent),

    /// An notice message.
    Notice(NoticeMessageEventContent),

    /// An text message.
    Text(TextMessageEventContent),

    /// An video message.
    Video(VideoMessageEventContent),
}

/// The payload of an audio message.
#[derive(Debug, Deserialize, Serialize)]
pub struct AudioMessageEventContent {
    /// The textual representation of this message.
    pub body: String,
    /// Metadata for the audio clip referred to in `url`.
    pub info: Option<AudioInfo>,
    /// The message type. Always *m.audio*.
    pub msgtype: MessageType,
    /// The URL to the audio clip.
    pub url: String,
}

/// Metadata about an audio clip.
#[derive(Debug, Deserialize, Serialize)]
pub struct AudioInfo {
    /// The duration of the audio in milliseconds.
    pub duration: Option<u64>,
    /// The mimetype of the audio, e.g. "audio/aac."
    pub mimetype: Option<String>,
    /// The size of the audio clip in bytes.
    pub size: Option<u64>,
}

/// The payload of an emote message.
#[derive(Debug, Deserialize, Serialize)]
pub struct EmoteMessageEventContent {
    /// The emote action to perform.
    pub body: String,
    /// The message type. Always *m.emote*.
    pub msgtype: MessageType,
}

/// The payload of a file message.
#[derive(Debug, Deserialize, Serialize)]
pub struct FileMessageEventContent {
    /// A human-readable description of the file. This is recommended to be the filename of the
    /// original upload.
    pub body: String,
    /// Metadata about the file referred to in `url`.
    pub info: Option<FileInfo>,
    /// The message type. Always *m.file*.
    pub msgtype: MessageType,
    /// Metadata about the image referred to in `thumbnail_url`.
    pub thumbnail_info: Option<ImageInfo>,
    /// The URL to the thumbnail of the file.
    pub thumbnail_url: Option<String>,
    /// The URL to the file.
    pub url: String,
}

/// Metadata about a file.
#[derive(Debug, Deserialize, Serialize)]
pub struct FileInfo {
    /// The mimetype of the file, e.g. "application/msword."
    pub mimetype: String,
    /// The size of the file in bytes.
    pub size: u64,
}

/// The payload of an image message.
#[derive(Debug, Deserialize, Serialize)]
pub struct ImageMessageEventContent {
    /// A textual representation of the image. This could be the alt text of the image, the filename
    /// of the image, or some kind of content description for accessibility e.g. "image attachment."
    pub body: String,
    /// Metadata about the image referred to in `url`.
    pub info: Option<ImageInfo>,
    /// The message type. Always *m.image*.
    pub msgtype: MessageType,
    /// Metadata about the image referred to in `thumbnail_url`.
    pub thumbnail_info: Option<ImageInfo>,
    /// The URL to the thumbnail of the image.
    pub thumbnail_url: Option<String>,
    /// The URL to the image.
    pub url: String,
}

/// The payload of a location message.
#[derive(Debug, Deserialize, Serialize)]
pub struct LocationMessageEventContent {
    /// A description of the location e.g. "Big Ben, London, UK,"or some kind of content description
    /// for accessibility, e.g. "location attachment."
    pub body: String,
    /// A geo URI representing the location.
    pub geo_uri: String,
    /// The message type. Always *m.location*.
    pub msgtype: MessageType,
    /// Metadata about the image referred to in `thumbnail_url`.
    pub thumbnail_info: Option<ImageInfo>,
    /// The URL to a thumbnail of the location being represented.
    pub thumbnail_url: Option<String>,
}

/// The payload of a notice message.
#[derive(Debug, Deserialize, Serialize)]
pub struct NoticeMessageEventContent {
    /// The notice text to send.
    pub body: String,
    /// The message type. Always *m.notice*.
    pub msgtype: MessageType,
}

/// The payload of a text message.
#[derive(Debug, Deserialize, Serialize)]
pub struct TextMessageEventContent {
    /// The body of the message.
    pub body: String,
    /// The message type. Always *m.text*.
    pub msgtype: MessageType,
}

/// The payload of a video message.
#[derive(Debug, Deserialize, Serialize)]
pub struct VideoMessageEventContent {
    /// A description of the video, e.g. "Gangnam Style," or some kind of content description for
    /// accessibility, e.g. "video attachment."
    pub body: String,
    /// Metadata about the video clip referred to in `url`.
    pub info: Option<VideoInfo>,
    /// The message type. Always *m.video*.
    pub msgtype: MessageType,
    /// The URL to the video clip.
    pub url: String,
}

/// Metadata about a video.
#[derive(Debug, Deserialize, Serialize)]
pub struct VideoInfo {
    /// The duration of the video in milliseconds.
    pub duration: Option<u64>,
    /// The height of the video in pixels.
    pub h: Option<u64>,
    /// The mimetype of the video, e.g. "video/mp4."
    pub mimetype: Option<String>,
    /// The size of the video in bytes.
    pub size: Option<u64>,
    /// Metadata about an image.
    pub thumbnail_info: Option<ImageInfo>,
    /// The URL to a thumbnail of the video clip.
    pub thumbnail_url: Option<String>,
    /// The width of the video in pixels.
    pub w: Option<u64>,
}

impl_enum! {
    MessageType {
        Audio => "audio",
        Emote => "emote",
        File => "file",
        Image => "image",
        Location => "location",
        Notice => "notice",
        Text => "text",
        Video => "video",
    }
}
