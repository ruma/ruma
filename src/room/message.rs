//! Types for the *m.room.message* event.

use std::{collections::BTreeMap, time::SystemTime};

use js_int::UInt;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{from_value, Value};

use super::{encrypted::EncryptedEventContent, EncryptedFile, ImageInfo, ThumbnailInfo};
use crate::{EventType, FromRaw};

pub mod feedback;

/// A message sent to a room.
#[derive(Clone, Debug, PartialEq, Serialize)]
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
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub unsigned: BTreeMap<String, Value>,
}

/// The payload for `MessageEvent`.
#[allow(clippy::large_enum_variant)]
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

    /// A server notice message.
    ServerNotice(ServerNoticeMessageEventContent),

    /// A text message.
    Text(TextMessageEventContent),

    /// A video message.
    Video(VideoMessageEventContent),

    /// A encrypted message.
    Encrypted(EncryptedEventContent),

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to ruma-events.
    #[doc(hidden)]
    __Nonexhaustive,
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
            __Nonexhaustive => {
                unreachable!("It should be impossible to obtain a __Nonexhaustive variant.")
            }
        }
    }
}

impl_room_event!(MessageEvent, MessageEventContent, EventType::RoomMessage);

impl Serialize for MessageEventContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::Error as _;

        match *self {
            MessageEventContent::Audio(ref content) => content.serialize(serializer),
            MessageEventContent::Emote(ref content) => content.serialize(serializer),
            MessageEventContent::File(ref content) => content.serialize(serializer),
            MessageEventContent::Image(ref content) => content.serialize(serializer),
            MessageEventContent::Location(ref content) => content.serialize(serializer),
            MessageEventContent::Notice(ref content) => content.serialize(serializer),
            MessageEventContent::ServerNotice(ref content) => content.serialize(serializer),
            MessageEventContent::Text(ref content) => content.serialize(serializer),
            MessageEventContent::Video(ref content) => content.serialize(serializer),
            MessageEventContent::Encrypted(ref content) => content.serialize(serializer),
            MessageEventContent::__Nonexhaustive => Err(S::Error::custom(
                "Attempted to deserialize __Nonexhaustive variant.",
            )),
        }
    }
}

pub(crate) mod raw {
    use super::*;

    /// A message sent to a room.
    #[derive(Clone, Debug, Deserialize, PartialEq)]
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
        pub unsigned: BTreeMap<String, Value>,
    }

    /// The payload for `MessageEvent`.
    #[allow(clippy::large_enum_variant)]
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

        /// A server notice message.
        ServerNotice(ServerNoticeMessageEventContent),

        /// An text message.
        Text(TextMessageEventContent),

        /// A video message.
        Video(VideoMessageEventContent),

        /// A video message.
        Encrypted(EncryptedEventContent),

        /// Additional variants may be added in the future and will not be considered breaking changes
        /// to ruma-events.
        #[doc(hidden)]
        __Nonexhaustive,
    }

    impl<'de> Deserialize<'de> for MessageEventContent {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            use serde::de::Error as _;

            let value: Value = Deserialize::deserialize(deserializer)?;

            let message_type_value = match value.get("msgtype") {
                Some(value) => value.clone(),
                None => return Err(D::Error::missing_field("msgtype")),
            };

            let message_type = match from_value::<MessageType>(message_type_value) {
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
                MessageType::ServerNotice => {
                    let content = match from_value::<ServerNoticeMessageEventContent>(value) {
                        Ok(content) => content,
                        Err(error) => return Err(D::Error::custom(error.to_string())),
                    };

                    Ok(MessageEventContent::ServerNotice(content))
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
                MessageType::__Nonexhaustive => Err(D::Error::custom(
                    "Attempted to deserialize __Nonexhaustive variant.",
                )),
            }
        }
    }
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

    /// A server notice.
    #[serde(rename = "m.server_notice")]
    ServerNotice,

    /// A text message.
    #[serde(rename = "m.text")]
    Text,

    /// A video message.
    #[serde(rename = "m.video")]
    Video,

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to ruma-events.
    #[doc(hidden)]
    #[serde(skip)]
    __Nonexhaustive,
}

/// The payload for an audio message.
#[derive(Clone, Debug, Deserialize, PartialEq)]
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
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
#[derive(Clone, Debug, Deserialize, PartialEq)]
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
#[derive(Clone, Debug, Deserialize, PartialEq)]
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
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FileInfo {
    /// The mimetype of the file, e.g. "application/msword."
    pub mimetype: Option<String>,

    /// The size of the file in bytes.
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
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ImageMessageEventContent {
    /// A textual representation of the image. This could be the alt text of the image, the filename
    /// of the image, or some kind of content description for accessibility e.g. "image attachment."
    pub body: String,

    /// Metadata about the image referred to in `url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<ImageInfo>,

    /// The URL to the image.  Required if the file is unencrypted. The URL (typically
    /// [MXC URI](https://matrix.org/docs/spec/client_server/r0.5.0#mxc-uri)) to the image.
    pub url: Option<String>,

    /// Required if image is encrypted. Information on the encrypted image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<EncryptedFile>,
}

/// The payload for a location message.
#[derive(Clone, Debug, Deserialize, PartialEq)]
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
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct NoticeMessageEventContent {
    /// The notice text to send.
    pub body: String,

    /// Information about related messages for
    /// [rich replies](https://matrix.org/docs/spec/client_server/r0.5.0#rich-replies).
    #[serde(rename = "m.relates_to")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<RelatesTo>,
}

/// The payload for a server notice message.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ServerNoticeMessageEventContent {
    /// A human-readable description of the notice.
    pub body: String,

    /// The type of notice being represented.
    pub server_notice_type: ServerNoticeType,

    /// A URI giving a contact method for the server administrator.
    ///
    /// Required if the notice type is `m.server_notice.usage_limit_reached`.
    pub admin_contact: Option<String>,

    /// The kind of usage limit the server has exceeded.
    ///
    /// Required if the notice type is `m.server_notice.usage_limit_reached`.
    pub limit_type: Option<LimitType>,
}

/// Types of server notices.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum ServerNoticeType {
    /// The server has exceeded some limit which requires the server administrator to intervene.
    #[serde(rename = "m.server_notice.usage_limit_reached")]
    UsageLimitReached,

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to ruma-events.
    #[doc(hidden)]
    #[serde(skip)]
    __Nonexhaustive,
}

/// Types of usage limits.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum LimitType {
    /// The server's number of active users in the last 30 days has exceeded the maximum.
    ///
    /// New connections are being refused by the server. What defines "active" is left as an
    /// implementation detail, however servers are encouraged to treat syncing users as "active".
    #[serde(rename = "monthly_active_user")]
    MonthlyActiveUser,

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to ruma-events.
    #[doc(hidden)]
    #[serde(skip)]
    __Nonexhaustive,
}

/// The payload for a text message.
#[derive(Clone, Debug, Deserialize, PartialEq)]
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
    #[serde(rename = "m.relates_to")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<RelatesTo>,
}

/// The payload for a video message.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct VideoMessageEventContent {
    /// A description of the video, e.g. "Gangnam Style," or some kind of content description for
    /// accessibility, e.g. "video attachment."
    pub body: String,

    /// Metadata about the video clip referred to in `url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<VideoInfo>,

    /// The URL to the video clip.  Required if the file is unencrypted. The URL (typically
    /// [MXC URI](https://matrix.org/docs/spec/client_server/r0.5.0#mxc-uri)) to the video clip.
    pub url: Option<String>,

    /// Required if video clip is encrypted. Information on the encrypted video clip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<EncryptedFile>,
}

/// Metadata about a video.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RelatesTo {
    /// Information about another message being replied to.
    #[serde(rename = "m.in_reply_to")]
    pub in_reply_to: InReplyTo,
}

/// Information about the event a "rich reply" is replying to.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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

impl Serialize for AudioMessageEventContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len = 2;

        if self.info.is_some() {
            len += 1;
        }

        if self.url.is_some() {
            len += 1;
        }

        if self.file.is_some() {
            len += 1;
        }

        let mut state = serializer.serialize_struct("AudioMessageEventContent", len)?;

        state.serialize_field("body", &self.body)?;

        if self.info.is_some() {
            state.serialize_field("info", &self.info)?;
        }

        state.serialize_field("msgtype", "m.audio")?;

        if self.url.is_some() {
            state.serialize_field("url", &self.url)?;
        }

        if self.file.is_some() {
            state.serialize_field("file", &self.file)?;
        }

        state.end()
    }
}

impl Serialize for EmoteMessageEventContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len = 2;

        if self.format.is_some() {
            len += 1;
        }

        if self.formatted_body.is_some() {
            len += 1;
        }

        let mut state = serializer.serialize_struct("EmoteMessageEventContent", len)?;

        state.serialize_field("body", &self.body)?;

        if self.format.is_some() {
            state.serialize_field("format", &self.format)?;
        }

        if self.formatted_body.is_some() {
            state.serialize_field("formatted_body", &self.formatted_body)?;
        }

        state.serialize_field("msgtype", "m.emote")?;

        state.end()
    }
}

impl Serialize for FileMessageEventContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len = 2;

        if self.filename.is_some() {
            len += 1;
        }

        if self.info.is_some() {
            len += 1;
        }

        if self.url.is_some() {
            len += 1;
        }

        if self.file.is_some() {
            len += 1;
        }

        let mut state = serializer.serialize_struct("FileMessageEventContent", len)?;

        state.serialize_field("body", &self.body)?;

        if self.filename.is_some() {
            state.serialize_field("filename", &self.filename)?;
        }

        state.serialize_field("msgtype", "m.file")?;

        if self.info.is_some() {
            state.serialize_field("info", &self.info)?;
        }

        if self.url.is_some() {
            state.serialize_field("url", &self.url)?;
        }

        if self.file.is_some() {
            state.serialize_field("file", &self.file)?;
        }

        state.end()
    }
}

impl Serialize for ImageMessageEventContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len = 2;

        if self.info.is_some() {
            len += 1;
        }

        if self.url.is_some() {
            len += 1;
        }

        if self.file.is_some() {
            len += 1;
        }

        let mut state = serializer.serialize_struct("ImageMessageEventContent", len)?;

        state.serialize_field("body", &self.body)?;
        state.serialize_field("msgtype", "m.image")?;

        if self.info.is_some() {
            state.serialize_field("info", &self.info)?;
        }

        if self.url.is_some() {
            state.serialize_field("url", &self.url)?;
        }

        if self.file.is_some() {
            state.serialize_field("file", &self.file)?;
        }

        state.end()
    }
}

impl Serialize for LocationMessageEventContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len = 3;

        if self.info.is_some() {
            len += 1;
        }

        let mut state = serializer.serialize_struct("LocationMessageEventContent", len)?;

        state.serialize_field("body", &self.body)?;
        state.serialize_field("geo_uri", &self.geo_uri)?;
        state.serialize_field("msgtype", "m.location")?;

        if self.info.is_some() {
            state.serialize_field("info", &self.info)?;
        }

        state.end()
    }
}

impl Serialize for NoticeMessageEventContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len = 2;

        if self.relates_to.is_some() {
            len += 1;
        }

        let mut state = serializer.serialize_struct("NoticeMessageEventContent", len)?;

        state.serialize_field("body", &self.body)?;
        state.serialize_field("msgtype", "m.notice")?;

        if self.relates_to.is_some() {
            state.serialize_field("m.relates_to", &self.relates_to)?;
        }

        state.end()
    }
}

impl Serialize for ServerNoticeMessageEventContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len = 3;

        if self.admin_contact.is_some() {
            len += 1;
        }

        if self.limit_type.is_some() {
            len += 1;
        }

        let mut state = serializer.serialize_struct("ServerNoticeMessageEventContent", len)?;

        state.serialize_field("body", &self.body)?;
        state.serialize_field("msgtype", "m.server_notice")?;
        state.serialize_field("server_notice_type", &self.server_notice_type)?;

        if self.admin_contact.is_some() {
            state.serialize_field("admin_contact", &self.admin_contact)?;
        }

        if self.limit_type.is_some() {
            state.serialize_field("limit_type", &self.limit_type)?;
        }

        state.end()
    }
}

impl Serialize for TextMessageEventContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len = 2;

        if self.format.is_some() {
            len += 1;
        }

        if self.formatted_body.is_some() {
            len += 1;
        }

        if self.relates_to.is_some() {
            len += 1;
        }

        let mut state = serializer.serialize_struct("TextMessageEventContent", len)?;

        state.serialize_field("body", &self.body)?;

        if self.format.is_some() {
            state.serialize_field("format", &self.format)?;
        }

        if self.formatted_body.is_some() {
            state.serialize_field("formatted_body", &self.formatted_body)?;
        }

        state.serialize_field("msgtype", "m.text")?;

        if self.relates_to.is_some() {
            state.serialize_field("m.relates_to", &self.relates_to)?;
        }

        state.end()
    }
}

impl Serialize for VideoMessageEventContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len = 2;

        if self.info.is_some() {
            len += 1;
        }

        if self.url.is_some() {
            len += 1;
        }

        if self.file.is_some() {
            len += 1;
        }

        let mut state = serializer.serialize_struct("VideoMessageEventContent", len)?;

        state.serialize_field("body", &self.body)?;
        state.serialize_field("msgtype", "m.video")?;

        if self.info.is_some() {
            state.serialize_field("info", &self.info)?;
        }

        if self.url.is_some() {
            state.serialize_field("url", &self.url)?;
        }

        if self.file.is_some() {
            state.serialize_field("file", &self.file)?;
        }

        state.end()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{AudioMessageEventContent, MessageEventContent};
    use crate::room::message::{InReplyTo, RelatesTo, TextMessageEventContent};
    use crate::EventResult;
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
        let message_event_content = MessageEventContent::Audio(AudioMessageEventContent {
            body: "test".to_string(),
            info: None,
            url: Some("http://example.com/audio.mp3".to_string()),
            file: None,
        });

        let json_data = json!({
            "body": "test",
            "msgtype": "m.audio",
            "url": "http://example.com/audio.mp3"
        });

        assert_eq!(
            from_json_value::<EventResult<MessageEventContent>>(json_data)
                .unwrap()
                .into_result()
                .unwrap(),
            message_event_content
        );
    }

    #[test]
    fn deserialization_failure() {
        let json_data = json!({
            "body": "test","msgtype": "m.location",
            "url": "http://example.com/audio.mp3"
        });
        assert!(
            from_json_value::<EventResult<MessageEventContent>>(json_data)
                .unwrap()
                .into_result()
                .is_err()
        );
    }
}
