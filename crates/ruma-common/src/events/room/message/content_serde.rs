//! `Deserialize` implementation for RoomMessageEventContent and MessageType.

use serde::{de, Deserialize};
use serde_json::value::RawValue as RawJsonValue;

#[cfg(feature = "unstable-msc3245")]
use super::VoiceContent;
#[cfg(feature = "unstable-msc3488")]
use super::{
    AssetContent, LocationContent, LocationInfo, LocationMessageEventContent,
    MilliSecondsSinceUnixEpoch,
};
#[cfg(feature = "unstable-msc3246")]
use super::{AudioContent, AudioInfo, AudioMessageEventContent};
#[cfg(feature = "unstable-msc3551")]
use super::{FileContent, FileInfo, FileMessageEventContent, MediaSource, MessageContent};
#[cfg(feature = "unstable-msc3552")]
use super::{ImageContent, ImageInfo, ImageMessageEventContent, ThumbnailContent};
use super::{MessageType, Relation, RoomMessageEventContent};
#[cfg(feature = "unstable-msc3553")]
use super::{VideoContent, VideoInfo, VideoMessageEventContent};
use crate::serde::from_raw_json_value;

impl<'de> Deserialize<'de> for RoomMessageEventContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let mut deserializer = serde_json::Deserializer::from_str(json.get());
        let relates_to =
            Option::<Relation>::deserialize(&mut deserializer).map_err(de::Error::custom)?;

        Ok(Self { msgtype: from_raw_json_value(&json)?, relates_to })
    }
}

/// Helper struct to determine the msgtype from a `serde_json::value::RawValue`
#[derive(Debug, Deserialize)]
struct MessageTypeDeHelper {
    /// The message type field
    msgtype: String,
}

impl<'de> Deserialize<'de> for MessageType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let MessageTypeDeHelper { msgtype } = from_raw_json_value(&json)?;

        Ok(match msgtype.as_ref() {
            "m.audio" => Self::Audio(from_raw_json_value(&json)?),
            "m.emote" => Self::Emote(from_raw_json_value(&json)?),
            "m.file" => Self::File(from_raw_json_value(&json)?),
            "m.image" => Self::Image(from_raw_json_value(&json)?),
            "m.location" => Self::Location(from_raw_json_value(&json)?),
            "m.notice" => Self::Notice(from_raw_json_value(&json)?),
            "m.server_notice" => Self::ServerNotice(from_raw_json_value(&json)?),
            "m.text" => Self::Text(from_raw_json_value(&json)?),
            "m.video" => Self::Video(from_raw_json_value(&json)?),
            "m.key.verification.request" => Self::VerificationRequest(from_raw_json_value(&json)?),
            _ => Self::_Custom(from_raw_json_value(&json)?),
        })
    }
}

/// Helper struct for deserializing `AudioMessageEventContent` with stable and unstable field names.
///
/// It's not possible to use the `alias` attribute of serde because of
/// https://github.com/serde-rs/serde/issues/1504.
#[derive(Clone, Debug, Deserialize)]
#[cfg(feature = "unstable-msc3246")]
pub struct AudioMessageEventContentDeHelper {
    /// The textual representation of this message.
    pub body: String,

    /// The source of the audio clip.
    #[serde(flatten)]
    pub source: MediaSource,

    /// Metadata for the audio clip referred to in `source`.
    pub info: Option<Box<AudioInfo>>,

    /// Extensible-event text representation of the message.
    #[serde(flatten)]
    pub message: Option<MessageContent>,

    /// Extensible-event file content of the message, with stable name.
    #[serde(rename = "m.file")]
    pub file_stable: Option<FileContent>,

    /// Extensible-event file content of the message, with unstable name.
    #[serde(rename = "org.matrix.msc1767.file")]
    pub file_unstable: Option<FileContent>,

    /// Extensible-event audio info of the message, with stable name.
    #[serde(rename = "m.audio")]
    pub audio_stable: Option<AudioContent>,

    /// Extensible-event audio info of the message, with unstable name.
    #[serde(rename = "org.matrix.msc1767.audio")]
    pub audio_unstable: Option<AudioContent>,

    /// Extensible-event voice flag of the message, with stable name.
    #[cfg(feature = "unstable-msc3245")]
    #[serde(rename = "m.voice")]
    pub voice_stable: Option<VoiceContent>,

    /// Extensible-event voice flag of the message, with unstable name.
    #[cfg(feature = "unstable-msc3245")]
    #[serde(rename = "org.matrix.msc3245.voice")]
    pub voice_unstable: Option<VoiceContent>,
}

#[cfg(feature = "unstable-msc3246")]
impl From<AudioMessageEventContentDeHelper> for AudioMessageEventContent {
    fn from(helper: AudioMessageEventContentDeHelper) -> Self {
        let AudioMessageEventContentDeHelper {
            body,
            source,
            info,
            message,
            file_stable,
            file_unstable,
            audio_stable,
            audio_unstable,
            #[cfg(feature = "unstable-msc3245")]
            voice_stable,
            #[cfg(feature = "unstable-msc3245")]
            voice_unstable,
        } = helper;

        let file = file_stable.or(file_unstable);
        let audio = audio_stable.or(audio_unstable);
        #[cfg(feature = "unstable-msc3245")]
        let voice = voice_stable.or(voice_unstable);

        Self {
            body,
            source,
            info,
            message,
            file,
            audio,
            #[cfg(feature = "unstable-msc3245")]
            voice,
        }
    }
}

/// Helper struct for deserializing `FileMessageEventContent` with stable and unstable field names.
///
/// It's not possible to use the `alias` attribute of serde because of
/// https://github.com/serde-rs/serde/issues/1504.
#[derive(Clone, Debug, Deserialize)]
#[cfg(feature = "unstable-msc3551")]
pub struct FileMessageEventContentDeHelper {
    /// A human-readable description of the file.
    pub body: String,

    /// The original filename of the uploaded file.
    pub filename: Option<String>,

    /// The source of the file.
    #[serde(flatten)]
    pub source: MediaSource,

    /// Metadata about the file referred to in `source`.
    pub info: Option<Box<FileInfo>>,

    /// Extensible-event text representation of the message.
    #[serde(flatten)]
    pub message: Option<MessageContent>,

    /// Extensible-event file content of the message, with stable name.
    #[serde(rename = "m.file")]
    pub file_stable: Option<FileContent>,

    /// Extensible-event file content of the message, with unstable name.
    #[serde(rename = "org.matrix.msc1767.file")]
    pub file_unstable: Option<FileContent>,
}

#[cfg(feature = "unstable-msc3551")]
impl From<FileMessageEventContentDeHelper> for FileMessageEventContent {
    fn from(helper: FileMessageEventContentDeHelper) -> Self {
        let FileMessageEventContentDeHelper {
            body,
            filename,
            source,
            info,
            message,
            file_stable,
            file_unstable,
        } = helper;

        let file = file_stable.or(file_unstable);

        Self { body, filename, source, info, message, file }
    }
}

/// Helper struct for deserializing `ImageMessageEventContent` with stable and unstable field names.
///
/// It's not possible to use the `alias` attribute of serde because of
/// https://github.com/serde-rs/serde/issues/1504.
#[derive(Clone, Debug, Deserialize)]
#[cfg(feature = "unstable-msc3552")]
pub struct ImageMessageEventContentDeHelper {
    /// A textual representation of the image.
    pub body: String,

    /// The source of the image.
    #[serde(flatten)]
    pub source: MediaSource,

    /// Metadata about the image referred to in `source`.
    pub info: Option<Box<ImageInfo>>,

    /// Extensible-event text representation of the message.
    #[serde(flatten)]
    pub message: Option<MessageContent>,

    /// Extensible-event file content of the message, with unstable name.
    #[serde(rename = "m.file")]
    pub file_stable: Option<FileContent>,

    /// Extensible-event file content of the message, with unstable name.
    #[serde(rename = "org.matrix.msc1767.file")]
    pub file_unstable: Option<FileContent>,

    /// Extensible-event image info of the message, with stable name.
    #[serde(rename = "m.image")]
    pub image_stable: Option<Box<ImageContent>>,

    /// Extensible-event image info of the message, with unstable name.
    #[serde(rename = "org.matrix.msc1767.image")]
    pub image_unstable: Option<Box<ImageContent>>,

    /// Extensible-event thumbnails of the message, with stable name.
    #[serde(rename = "m.thumbnail")]
    pub thumbnail_stable: Option<Vec<ThumbnailContent>>,

    /// Extensible-event thumbnails of the message, with unstable name.
    #[serde(rename = "org.matrix.msc1767.thumbnail")]
    pub thumbnail_unstable: Option<Vec<ThumbnailContent>>,

    /// Extensible-event captions of the message, with stable name.
    #[serde(rename = "m.caption")]
    pub caption_stable: Option<MessageContent>,

    /// Extensible-event captions of the message, with unstable name.
    #[serde(rename = "org.matrix.msc1767.caption")]
    pub caption_unstable: Option<MessageContent>,
}

#[cfg(feature = "unstable-msc3552")]
impl From<ImageMessageEventContentDeHelper> for ImageMessageEventContent {
    fn from(helper: ImageMessageEventContentDeHelper) -> Self {
        let ImageMessageEventContentDeHelper {
            body,
            source,
            info,
            message,
            file_stable,
            file_unstable,
            image_stable,
            image_unstable,
            thumbnail_stable,
            thumbnail_unstable,
            caption_stable,
            caption_unstable,
        } = helper;

        let file = file_stable.or(file_unstable);
        let image = image_stable.or(image_unstable);
        let thumbnail = thumbnail_stable.or(thumbnail_unstable);
        let caption = caption_stable.or(caption_unstable);

        Self { body, source, info, message, file, image, thumbnail, caption }
    }
}

/// Helper struct for deserializing `LocationMessageEventContent` with stable and unstable field
/// names.
///
/// It's not possible to use the `alias` attribute of serde because of
/// https://github.com/serde-rs/serde/issues/1504.
#[derive(Clone, Debug, Deserialize)]
#[cfg(feature = "unstable-msc3488")]
pub struct LocationMessageEventContentDeHelper {
    /// A description of the location.
    pub body: String,

    /// A geo URI representing the location.
    pub geo_uri: String,

    /// Info about the location being represented.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<LocationInfo>>,

    /// Extensible-event text representation of the message.
    #[serde(flatten)]
    pub message: Option<MessageContent>,

    /// Extensible-event location info of the message, with stable name.
    #[serde(rename = "m.location")]
    pub location_stable: Option<LocationContent>,

    /// Extensible-event location info of the message, with unstable name.
    #[serde(rename = "org.matrix.msc3488.location")]
    pub location_unstable: Option<LocationContent>,

    /// Extensible-event asset this message refers to, with stable name.
    #[serde(rename = "m.asset")]
    pub asset_stable: Option<AssetContent>,

    /// Extensible-event asset this message refers to, with unstable name.
    #[serde(rename = "org.matrix.msc3488.asset")]
    pub asset_unstable: Option<AssetContent>,

    /// Extensible-event timestamp this message refers to, with stable name.
    #[serde(rename = "m.ts")]
    pub ts_stable: Option<MilliSecondsSinceUnixEpoch>,

    /// Extensible-event timestamp this message refers to, with unstable name.
    #[serde(rename = "org.matrix.msc3488.ts")]
    pub ts_unstable: Option<MilliSecondsSinceUnixEpoch>,
}

#[cfg(feature = "unstable-msc3488")]
impl From<LocationMessageEventContentDeHelper> for LocationMessageEventContent {
    fn from(helper: LocationMessageEventContentDeHelper) -> Self {
        let LocationMessageEventContentDeHelper {
            body,
            geo_uri,
            info,
            message,
            location_stable,
            location_unstable,
            asset_stable,
            asset_unstable,
            ts_stable,
            ts_unstable,
        } = helper;

        let location = location_stable.or(location_unstable);
        let asset = asset_stable.or(asset_unstable);
        let ts = ts_stable.or(ts_unstable);

        Self { body, geo_uri, info, message, location, asset, ts }
    }
}

/// Helper struct for deserializing `VideoMessageEventContent` with stable and unstable field names.
///
/// It's not possible to use the `alias` attribute of serde because of
/// https://github.com/serde-rs/serde/issues/1504.
#[derive(Clone, Debug, Deserialize)]
#[cfg(feature = "unstable-msc3553")]
pub struct VideoMessageEventContentDeHelper {
    /// A description of the video.
    pub body: String,

    /// The source of the video clip.
    #[serde(flatten)]
    pub source: MediaSource,

    /// Metadata about the video clip referred to in `source`.
    pub info: Option<Box<VideoInfo>>,

    /// Extensible-event text representation of the message.
    #[serde(flatten)]
    pub message: Option<MessageContent>,

    /// Extensible-event file content of the message, with stable name.
    #[serde(rename = "m.file")]
    pub file_stable: Option<FileContent>,

    /// Extensible-event file content of the message, with unstable name.
    #[serde(rename = "org.matrix.msc1767.file")]
    pub file_unstable: Option<FileContent>,

    /// Extensible-event video info of the message, with stable name.
    #[serde(rename = "m.video")]
    pub video_stable: Option<Box<VideoContent>>,

    /// Extensible-event video info of the message, with unstable name.
    #[serde(rename = "org.matrix.msc1767.video")]
    pub video_unstable: Option<Box<VideoContent>>,

    /// Extensible-event thumbnails of the message, with stable name.
    #[serde(rename = "m.thumbnail")]
    pub thumbnail_stable: Option<Vec<ThumbnailContent>>,

    /// Extensible-event thumbnails of the message, with unstable name.
    #[serde(rename = "org.matrix.msc1767.thumbnail")]
    pub thumbnail_unstable: Option<Vec<ThumbnailContent>>,

    /// Extensible-event captions of the message, with stable name.
    #[serde(rename = "m.caption")]
    pub caption_stable: Option<MessageContent>,

    /// Extensible-event captions of the message, with unstable name.
    #[serde(rename = "org.matrix.msc1767.caption")]
    pub caption_unstable: Option<MessageContent>,
}

#[cfg(feature = "unstable-msc3553")]
impl From<VideoMessageEventContentDeHelper> for VideoMessageEventContent {
    fn from(helper: VideoMessageEventContentDeHelper) -> Self {
        let VideoMessageEventContentDeHelper {
            body,
            source,
            info,
            message,
            file_stable,
            file_unstable,
            video_stable,
            video_unstable,
            thumbnail_stable,
            thumbnail_unstable,
            caption_stable,
            caption_unstable,
        } = helper;

        let file = file_stable.or(file_unstable);
        let video = video_stable.or(video_unstable);
        let thumbnail = thumbnail_stable.or(thumbnail_unstable);
        let caption = caption_stable.or(caption_unstable);

        Self { body, source, info, message, file, video, thumbnail, caption }
    }
}
