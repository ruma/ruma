//! Types for the [`m.sticker`] event.
//!
//! [`m.sticker`]: https://spec.matrix.org/latest/client-server-api/#msticker

use ruma_common::OwnedMxcUri;
use ruma_macros::EventContent;
use serde::{de, Deserialize, Serialize};

#[cfg(feature = "compat-encrypted-stickers")]
use crate::room::EncryptedFile;
use crate::room::{ImageInfo, MediaSource};

/// The source of a sticker media file.
#[derive(Clone, Debug, Serialize)]
#[allow(clippy::exhaustive_enums)]
pub enum StickerMediaSource {
    /// The MXC URI to the unencrypted media file.
    #[serde(rename = "url")]
    Plain(OwnedMxcUri),

    /// The encryption info of the encrypted media file.
    #[cfg(feature = "compat-encrypted-stickers")]
    #[serde(rename = "file")]
    Encrypted(Box<EncryptedFile>),
}

// Custom implementation of `Deserialize`, because serde doesn't guarantee what variant will be
// deserialized for "externally tagged"¹ enums where multiple "tag" fields exist.
//
// ¹ https://serde.rs/enum-representations.html
impl<'de> Deserialize<'de> for StickerMediaSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct StickerMediaSourceJsonRepr {
            url: Option<OwnedMxcUri>,
            #[cfg(feature = "compat-encrypted-stickers")]
            file: Option<Box<EncryptedFile>>,
        }

        match StickerMediaSourceJsonRepr::deserialize(deserializer)? {
            #[cfg(feature = "compat-encrypted-stickers")]
            StickerMediaSourceJsonRepr { url: None, file: None } => {
                Err(de::Error::missing_field("url"))
            }
            #[cfg(not(feature = "compat-encrypted-stickers"))]
            StickerMediaSourceJsonRepr { url: None } => Err(de::Error::missing_field("url")),
            // Prefer file if it is set
            #[cfg(feature = "compat-encrypted-stickers")]
            StickerMediaSourceJsonRepr { file: Some(file), .. } => {
                Ok(StickerMediaSource::Encrypted(file))
            }
            StickerMediaSourceJsonRepr { url: Some(url), .. } => Ok(StickerMediaSource::Plain(url)),
        }
    }
}
impl From<StickerMediaSource> for MediaSource {
    fn from(value: StickerMediaSource) -> Self {
        match value {
            StickerMediaSource::Plain(url) => MediaSource::Plain(url),
            #[cfg(feature = "compat-encrypted-stickers")]
            StickerMediaSource::Encrypted(file) => MediaSource::Encrypted(file),
        }
    }
}

/// The content of an `m.sticker` event.
///
/// A sticker message.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.sticker", kind = MessageLike)]
pub struct StickerEventContent {
    /// A textual representation or associated description of the sticker image.
    ///
    /// This could be the alt text of the original image, or a message to accompany and further
    /// describe the sticker.
    pub body: String,

    /// Metadata about the image referred to in `url` including a thumbnail representation.
    pub info: ImageInfo,

    /// The media source of the sticker image.
    #[serde(flatten)]
    pub source: StickerMediaSource,
}

impl StickerEventContent {
    /// Creates a new `StickerEventContent` with the given body, image info and URL.
    pub fn new(body: String, info: ImageInfo, url: OwnedMxcUri) -> Self {
        Self { body, info, source: StickerMediaSource::Plain(url.clone()) }
    }

    /// Creates a new `StickerEventContent` with the given body, image info, URL, and media source.
    #[cfg(feature = "compat-encrypted-stickers")]
    pub fn with_source(body: String, info: ImageInfo, source: StickerMediaSource) -> Self {
        Self { body, info, source }
    }
}
