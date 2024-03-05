use std::time::Duration;

use js_int::UInt;
use ruma_common::OwnedMxcUri;
use serde::{Deserialize, Serialize};

use super::FormattedBody;
use crate::room::{EncryptedFile, MediaSource, ThumbnailInfo};

/// The payload for a video message.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.video")]
pub struct VideoMessageEventContent {
    /// A description of the video.
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

    /// The source of the video clip.
    #[serde(flatten)]
    pub source: MediaSource,

    /// Metadata about the video clip referred to in `source`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<VideoInfo>>,
}

impl VideoMessageEventContent {
    /// Creates a new `VideoMessageEventContent` with the given body and source.
    pub fn new(body: String, source: MediaSource) -> Self {
        Self { body, formatted: None, filename: None, source, info: None }
    }

    /// Creates a new non-encrypted `VideoMessageEventContent` with the given body and url.
    pub fn plain(body: String, url: OwnedMxcUri) -> Self {
        Self::new(body, MediaSource::Plain(url))
    }

    /// Creates a new encrypted `VideoMessageEventContent` with the given body and encrypted
    /// file.
    pub fn encrypted(body: String, file: EncryptedFile) -> Self {
        Self::new(body, MediaSource::Encrypted(Box::new(file)))
    }

    /// Creates a new `VideoMessageEventContent` from `self` with the `info` field set to the given
    /// value.
    ///
    /// Since the field is public, you can also assign to it directly. This method merely acts
    /// as a shorthand for that, because it is very common to set this field.
    pub fn info(self, info: impl Into<Option<Box<VideoInfo>>>) -> Self {
        Self { info: info.into(), ..self }
    }
}

/// Metadata about a video.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct VideoInfo {
    /// The duration of the video in milliseconds.
    #[serde(
        with = "ruma_common::serde::duration::opt_ms",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub duration: Option<Duration>,

    /// The height of the video in pixels.
    #[serde(rename = "h", skip_serializing_if = "Option::is_none")]
    pub height: Option<UInt>,

    /// The width of the video in pixels.
    #[serde(rename = "w", skip_serializing_if = "Option::is_none")]
    pub width: Option<UInt>,

    /// The mimetype of the video, e.g. "video/mp4".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// The size of the video in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<UInt>,

    /// Metadata about the image referred to in `thumbnail_source`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_info: Option<Box<ThumbnailInfo>>,

    /// The source of the thumbnail of the video clip.
    #[serde(
        flatten,
        with = "crate::room::thumbnail_source_serde",
        skip_serializing_if = "Option::is_none"
    )]
    pub thumbnail_source: Option<MediaSource>,

    /// The [BlurHash](https://blurha.sh) for this video.
    ///
    /// This uses the unstable prefix in
    /// [MSC2448](https://github.com/matrix-org/matrix-spec-proposals/pull/2448).
    #[cfg(feature = "unstable-msc2448")]
    #[serde(rename = "xyz.amorgan.blurhash", skip_serializing_if = "Option::is_none")]
    pub blurhash: Option<String>,
}

impl VideoInfo {
    /// Creates an empty `VideoInfo`.
    pub fn new() -> Self {
        Self::default()
    }
}
