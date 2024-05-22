use js_int::UInt;
use ruma_common::OwnedMxcUri;
use serde::{Deserialize, Serialize};

use super::FormattedBody;
use crate::room::{EncryptedFile, MediaSource, ThumbnailInfo};

/// The payload for a file message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.file")]
pub struct FileMessageEventContent {
    /// A human-readable description of the file.
    ///
    /// If the `filename` field is not set or has the same value, this is the filename of the
    /// uploaded file. Otherwise, this should be interpreted as a user-written media caption.
    pub body: String,

    /// Formatted form of the message `body`.
    ///
    /// This should only be set if the body represents a caption.
    #[serde(flatten)]
    pub formatted: Option<FormattedBody>,

    /// The original filename of the uploaded file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,

    /// The source of the file.
    #[serde(flatten)]
    pub source: MediaSource,

    /// Metadata about the file referred to in `source`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<FileInfo>>,
}

impl FileMessageEventContent {
    /// Creates a new `FileMessageEventContent` with the given body and source.
    pub fn new(body: String, source: MediaSource) -> Self {
        Self { body, formatted: None, filename: None, source, info: None }
    }

    /// Creates a new non-encrypted `FileMessageEventContent` with the given body and url.
    pub fn plain(body: String, url: OwnedMxcUri) -> Self {
        Self::new(body, MediaSource::Plain(url))
    }

    /// Creates a new encrypted `FileMessageEventContent` with the given body and encrypted
    /// file.
    pub fn encrypted(body: String, file: EncryptedFile) -> Self {
        Self::new(body, MediaSource::Encrypted(Box::new(file)))
    }

    /// Creates a new `FileMessageEventContent` from `self` with the `info` field set to the given
    /// value.
    ///
    /// Since the field is public, you can also assign to it directly. This method merely acts
    /// as a shorthand for that, because it is very common to set this field.
    pub fn info(self, info: impl Into<Option<Box<FileInfo>>>) -> Self {
        Self { info: info.into(), ..self }
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
        with = "crate::room::thumbnail_source_serde",
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
