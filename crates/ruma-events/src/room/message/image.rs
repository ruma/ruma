use ruma_common::OwnedMxcUri;
use serde::{Deserialize, Serialize};

use super::FormattedBody;
use crate::room::{
    message::media_caption::{caption, formatted_caption},
    EncryptedFile, ImageInfo, MediaSource,
};

/// The payload for an image message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct ImageMessageEventContent {
    /// A textual representation of the image.
    ///
    /// If the `filename` field is not set or has the same value, this is the filename of the
    /// uploaded file. Otherwise, this should be interpreted as a user-written media caption.
    pub body: String,

    /// Formatted form of the message `body`.
    ///
    /// This should only be set if the body represents a caption.
    #[serde(flatten)]
    pub formatted: Option<FormattedBody>,

    /// The original filename of the uploaded file as deserialized from the event.
    ///
    /// It is recommended to use the `filename` method to get the filename which automatically
    /// falls back to the `body` field when the `filename` field is not set.
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

    /// Creates a new `ImageMessageEventContent` from `self` with the `info` field set to the given
    /// value.
    ///
    /// Since the field is public, you can also assign to it directly. This method merely acts
    /// as a shorthand for that, because it is very common to set this field.
    pub fn info(self, info: impl Into<Option<Box<ImageInfo>>>) -> Self {
        Self { info: info.into(), ..self }
    }

    /// Computes the filename of the image as defined by the [spec](https://spec.matrix.org/latest/client-server-api/#media-captions).
    ///
    /// This differs from the `filename` field as this method falls back to the `body` field when
    /// the `filename` field is not set.
    pub fn filename(&self) -> &str {
        self.filename.as_deref().unwrap_or(&self.body)
    }

    /// Returns the caption for the image as defined by the [spec](https://spec.matrix.org/latest/client-server-api/#media-captions).
    ///
    /// In short, this is the `body` field if the `filename` field exists and has a different value,
    /// otherwise the media file does not have a caption.
    pub fn caption(&self) -> Option<&str> {
        caption(&self.body, self.filename.as_deref())
    }

    /// Returns the formatted caption for the image as defined by the [spec](https://spec.matrix.org/latest/client-server-api/#media-captions).
    ///
    /// This is the same as `caption`, but returns the formatted body instead of the plain body.
    pub fn formatted_caption(&self) -> Option<&FormattedBody> {
        formatted_caption(&self.body, self.formatted.as_ref(), self.filename.as_deref())
    }
}
