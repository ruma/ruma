//! Types for extensible image message events ([MSC3552]).
//!
//! [MSC3552]: https://github.com/matrix-org/matrix-spec-proposals/pull/3552

use std::ops::Deref;

use js_int::UInt;
use ruma_common::OwnedMxcUri;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{
    file::{CaptionContentBlock, EncryptedContent, FileContentBlock},
    message::TextContentBlock,
    room::message::Relation,
};

/// The payload for an extensible image message.
///
/// This is the new primary type introduced in [MSC3552] and should only be sent in rooms with a
/// version that supports it. This type replaces both the `m.room.message` type with `msgtype:
/// "m.image"` and the `m.sticker` type. To replace the latter, `sticker` must be set to `true` in
/// `image_details`. See the documentation of the [`message`] module for more information.
///
/// [MSC3552]: https://github.com/matrix-org/matrix-spec-proposals/pull/3552
/// [`message`]: super::message
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc1767.image", kind = MessageLike, without_relation)]
pub struct ImageEventContent {
    /// The text representation of the message.
    #[serde(rename = "org.matrix.msc1767.text")]
    pub text: TextContentBlock,

    /// The file content of the message.
    #[serde(rename = "org.matrix.msc1767.file")]
    pub file: FileContentBlock,

    /// The image details of the message, if any.
    #[serde(rename = "org.matrix.msc1767.image_details", skip_serializing_if = "Option::is_none")]
    pub image_details: Option<ImageDetailsContentBlock>,

    /// The thumbnails of the message, if any.
    ///
    /// This is optional and defaults to an empty array.
    #[serde(
        rename = "org.matrix.msc1767.thumbnail",
        default,
        skip_serializing_if = "ThumbnailContentBlock::is_empty"
    )]
    pub thumbnail: ThumbnailContentBlock,

    /// The caption of the message, if any.
    #[serde(rename = "org.matrix.msc1767.caption", skip_serializing_if = "Option::is_none")]
    pub caption: Option<CaptionContentBlock>,

    /// The alternative text of the image, for accessibility considerations, if any.
    #[serde(rename = "org.matrix.msc1767.alt_text", skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<AltTextContentBlock>,

    /// Whether this message is automated.
    #[cfg(feature = "unstable-msc3955")]
    #[serde(
        default,
        skip_serializing_if = "ruma_common::serde::is_default",
        rename = "org.matrix.msc1767.automated"
    )]
    pub automated: bool,

    /// Information about related messages.
    #[serde(
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::room::message::relation_serde::deserialize_relation"
    )]
    pub relates_to: Option<Relation<ImageEventContentWithoutRelation>>,
}

impl ImageEventContent {
    /// Creates a new `ImageEventContent` with the given fallback representation and
    /// file.
    pub fn new(text: TextContentBlock, file: FileContentBlock) -> Self {
        Self {
            text,
            file,
            image_details: None,
            thumbnail: Default::default(),
            caption: None,
            alt_text: None,
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
            relates_to: None,
        }
    }

    /// Creates a new `ImageEventContent` with the given plain text fallback representation and
    /// file.
    pub fn with_plain_text(plain_text: impl Into<String>, file: FileContentBlock) -> Self {
        Self {
            text: TextContentBlock::plain(plain_text),
            file,
            image_details: None,
            thumbnail: Default::default(),
            caption: None,
            alt_text: None,
            #[cfg(feature = "unstable-msc3955")]
            automated: false,
            relates_to: None,
        }
    }
}

/// A block for details of image content.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ImageDetailsContentBlock {
    /// The height of the image in pixels.
    pub height: UInt,

    /// The width of the image in pixels.
    pub width: UInt,

    /// Whether the image should be presented as sticker.
    #[serde(
        rename = "org.matrix.msc1767.sticker",
        default,
        skip_serializing_if = "ruma_common::serde::is_default"
    )]
    pub sticker: bool,
}

impl ImageDetailsContentBlock {
    /// Creates a new `ImageDetailsContentBlock` with the given width and height.
    pub fn new(width: UInt, height: UInt) -> Self {
        Self { height, width, sticker: Default::default() }
    }
}

/// A block for thumbnail content.
///
/// This is an array of [`Thumbnail`].
///
/// To construct a `ThumbnailContentBlock` convert a `Vec<Thumbnail>` with
/// `ThumbnailContentBlock::from()` / `.into()`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[allow(clippy::exhaustive_structs)]
pub struct ThumbnailContentBlock(Vec<Thumbnail>);

impl ThumbnailContentBlock {
    /// Whether this content block is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<Thumbnail>> for ThumbnailContentBlock {
    fn from(thumbnails: Vec<Thumbnail>) -> Self {
        Self(thumbnails)
    }
}

impl FromIterator<Thumbnail> for ThumbnailContentBlock {
    fn from_iter<T: IntoIterator<Item = Thumbnail>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Deref for ThumbnailContentBlock {
    type Target = [Thumbnail];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Thumbnail content.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Thumbnail {
    /// The file info of the thumbnail.
    #[serde(rename = "org.matrix.msc1767.file")]
    pub file: ThumbnailFileContentBlock,

    /// The image info of the thumbnail.
    #[serde(rename = "org.matrix.msc1767.image_details")]
    pub image_details: ThumbnailImageDetailsContentBlock,
}

impl Thumbnail {
    /// Creates a `Thumbnail` with the given file and image details.
    pub fn new(
        file: ThumbnailFileContentBlock,
        image_details: ThumbnailImageDetailsContentBlock,
    ) -> Self {
        Self { file, image_details }
    }
}

/// A block for thumbnail file content.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ThumbnailFileContentBlock {
    /// The URL to the thumbnail.
    pub url: OwnedMxcUri,

    /// The mimetype of the file, e.g. "image/png".
    pub mimetype: String,

    /// The original filename of the uploaded file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The size of the file in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<UInt>,

    /// Information on the encrypted thumbnail.
    ///
    /// Required if the thumbnail is encrypted.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub encryption_info: Option<Box<EncryptedContent>>,
}

impl ThumbnailFileContentBlock {
    /// Creates a new non-encrypted `ThumbnailFileContentBlock` with the given url and mimetype.
    pub fn plain(url: OwnedMxcUri, mimetype: String) -> Self {
        Self { url, mimetype, name: None, size: None, encryption_info: None }
    }

    /// Creates a new encrypted `ThumbnailFileContentBlock` with the given url, mimetype and
    /// encryption info.
    pub fn encrypted(
        url: OwnedMxcUri,
        mimetype: String,
        encryption_info: EncryptedContent,
    ) -> Self {
        Self {
            url,
            mimetype,
            name: None,
            size: None,
            encryption_info: Some(Box::new(encryption_info)),
        }
    }

    /// Whether the thumbnail file is encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.encryption_info.is_some()
    }
}

/// A block for details of thumbnail image content.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ThumbnailImageDetailsContentBlock {
    /// The height of the image in pixels.
    pub height: UInt,

    /// The width of the image in pixels.
    pub width: UInt,
}

impl ThumbnailImageDetailsContentBlock {
    /// Creates a new `ThumbnailImageDetailsContentBlock` with the given width and height.
    pub fn new(width: UInt, height: UInt) -> Self {
        Self { height, width }
    }
}

/// A block for alternative text content.
///
/// The content should only contain plain text messages. Non-plain text messages should be ignored.
///
/// To construct an `AltTextContentBlock` with a custom [`TextContentBlock`], convert it with
/// `AltTextContentBlock::from()` / `.into()`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct AltTextContentBlock {
    /// The alternative text.
    #[serde(rename = "org.matrix.msc1767.text")]
    pub text: TextContentBlock,
}

impl AltTextContentBlock {
    /// A convenience constructor to create a plain text alternative text content block.
    pub fn plain(body: impl Into<String>) -> Self {
        Self { text: TextContentBlock::plain(body) }
    }
}

impl From<TextContentBlock> for AltTextContentBlock {
    fn from(text: TextContentBlock) -> Self {
        Self { text }
    }
}
