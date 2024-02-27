use ruma_common::OwnedMxcUri;
use serde::{Deserialize, Serialize};

use super::FormattedBody;
use crate::room::{EncryptedFile, ImageInfo, MediaSource};

/// The payload for an image message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.image")]
pub struct ImageMessageEventContent {
    /// A textual representation of the image.
    ///
    /// If the `filename` field is not set or has the same value, this is the filename of the
    /// uploaded file. Otherwise, this should be interpreted as a user-written media caption.
    pub body: String,

    /// Formatted form of the message `body`, if `body` is a caption.
    #[serde(flatten)]
    pub formatted: Option<FormattedBody>,

    /// The original filename of the uploaded file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,

    /// The source of the image.
    #[serde(flatten)]
    pub source: MediaSource,

    /// Metadata about the image referred to in `source`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<ImageInfo>>,
}

impl ImageMessageEventContent {
    /// Creates a new `ImageMessageEventContent` with the given body and source.
    pub fn new(body: String, source: MediaSource) -> Self {
        Self { body, formatted: None, filename: None, source, info: None }
    }

    /// Creates a new non-encrypted `ImageMessageEventContent` with the given body and url.
    pub fn plain(body: String, url: OwnedMxcUri) -> Self {
        Self::new(body, MediaSource::Plain(url))
    }

    /// Creates a new encrypted `ImageMessageEventContent` with the given body and encrypted
    /// file.
    pub fn encrypted(body: String, file: EncryptedFile) -> Self {
        Self::new(body, MediaSource::Encrypted(Box::new(file)))
    }

    /// Creates a new `ImageMessageEventContent` from `self` with the `filename` field set to the
    /// given value.
    ///
    /// Since the field is public, you can also assign to it directly. This method merely acts
    /// as a shorthand for that, because it is very common to set this field.
    pub fn filename(self, filename: impl Into<Option<String>>) -> Self {
        Self { filename: filename.into(), ..self }
    }

    /// Creates a new `ImageMessageEventContent` from `self` with the `formatted` field set to the
    /// given value.
    ///
    /// Since the field is public, you can also assign to it directly. This method merely acts
    /// as a shorthand for that, because it is very common to set this field.
    pub fn formatted(self, formatted: impl Into<Option<FormattedBody>>) -> Self {
        Self { formatted: formatted.into(), ..self }
    }

    /// Creates a new `ImageMessageEventContent` from `self` with the `info` field set to the given
    /// value.
    ///
    /// Since the field is public, you can also assign to it directly. This method merely acts
    /// as a shorthand for that, because it is very common to set this field.
    pub fn info(self, info: impl Into<Option<Box<ImageInfo>>>) -> Self {
        Self { info: info.into(), ..self }
    }
}
