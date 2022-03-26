//! `Deserialize` implementation for RoomMessageEventContent and MessageType.

use serde::{de, Deserialize};
use serde_json::value::RawValue as RawJsonValue;

#[cfg(feature = "unstable-msc3551")]
use super::{FileContent, FileInfo, FileMessageEventContent, MediaSource, MessageContent};
use super::{MessageType, Relation, RoomMessageEventContent};
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

    /// Metadata about the file referred to in `url`.
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
