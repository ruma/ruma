use js_int::UInt;
use serde::{Deserialize, Serialize};

#[cfg(feature = "unstable-msc3551")]
use crate::events::{
    file::{FileContent, FileContentInfo},
    message::MessageContent,
};
use crate::{
    events::room::{EncryptedFile, MediaSource, ThumbnailInfo},
    OwnedMxcUri,
};

/// The payload for a file message.
///
/// With the `unstable-msc3551` feature, this type contains the transitional format of
/// [`FileEventContent`]. See the documentation of the [`message`] module for more information.
///
/// [`FileEventContent`]: crate::events::file::FileEventContent
/// [`message`]: crate::events::message
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.file")]
#[cfg_attr(
    feature = "unstable-msc3551",
    serde(from = "super::content_serde::FileMessageEventContentDeHelper")
)]
pub struct FileMessageEventContent {
    /// A human-readable description of the file.
    ///
    /// This is recommended to be the filename of the original upload.
    pub body: String,

    /// The original filename of the uploaded file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,

    /// The source of the file.
    #[serde(flatten)]
    pub source: MediaSource,

    /// Metadata about the file referred to in `source`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<FileInfo>>,

    /// Extensible-event text representation of the message.
    ///
    /// If present, this should be preferred over the `body` field.
    #[cfg(feature = "unstable-msc3551")]
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub message: Option<MessageContent>,

    /// Extensible-event file content of the message.
    ///
    /// If present, this should be preferred over the `source` and `info` fields.
    #[cfg(feature = "unstable-msc3551")]
    #[serde(rename = "org.matrix.msc1767.file", skip_serializing_if = "Option::is_none")]
    pub file: Option<FileContent>,
}

impl FileMessageEventContent {
    /// Creates a new non-encrypted `FileMessageEventContent` with the given body, url and
    /// optional extra info.
    pub fn plain(body: String, url: OwnedMxcUri, info: Option<Box<FileInfo>>) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3551")]
            message: Some(MessageContent::plain(body.clone())),
            #[cfg(feature = "unstable-msc3551")]
            file: Some(FileContent::plain(
                url.clone(),
                info.as_deref().map(|info| Box::new(info.into())),
            )),
            body,
            filename: None,
            source: MediaSource::Plain(url),
            info,
        }
    }

    /// Creates a new encrypted `FileMessageEventContent` with the given body and encrypted
    /// file.
    pub fn encrypted(body: String, file: EncryptedFile) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3551")]
            message: Some(MessageContent::plain(body.clone())),
            #[cfg(feature = "unstable-msc3551")]
            file: Some(FileContent::encrypted(file.url.clone(), (&file).into(), None)),
            body,
            filename: None,
            source: MediaSource::Encrypted(Box::new(file)),
            info: None,
        }
    }

    /// Create a new `FileMessageEventContent` with the given message and file info.
    #[cfg(feature = "unstable-msc3551")]
    pub fn from_extensible_content(message: MessageContent, file: FileContent) -> Self {
        let body = if let Some(body) = message.find_plain() {
            body.to_owned()
        } else {
            message[0].body.clone()
        };
        let filename = file.info.as_deref().and_then(|info| info.name.clone());
        let info = file.info.as_deref().map(|info| Box::new(info.into()));
        let source = (&file).into();

        Self { message: Some(message), file: Some(file), body, filename, source, info }
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

    /// Metadata about the image referred to in `thumbnail_source`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_info: Option<Box<ThumbnailInfo>>,

    /// The source of the thumbnail of the file.
    #[serde(
        flatten,
        with = "crate::events::room::thumbnail_source_serde",
        skip_serializing_if = "Option::is_none"
    )]
    pub thumbnail_source: Option<MediaSource>,
}

impl FileInfo {
    /// Creates an empty `FileInfo`.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(feature = "unstable-msc3551")]
impl From<&FileContentInfo> for FileInfo {
    fn from(info: &FileContentInfo) -> Self {
        let FileContentInfo { mimetype, size, .. } = info;
        Self { mimetype: mimetype.to_owned(), size: size.to_owned(), ..Default::default() }
    }
}
