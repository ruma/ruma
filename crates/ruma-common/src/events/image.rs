//! Types for extensible image message events ([MSC3552]).
//!
//! [MSC3552]: https://github.com/matrix-org/matrix-spec-proposals/pull/3552

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{
    file::{EncryptedContent, FileContent},
    message::MessageContent,
    room::{
        message::{ImageMessageEventContent, Relation},
        ImageInfo, MediaSource, ThumbnailInfo,
    },
};
use crate::OwnedMxcUri;

/// The payload for an extensible image message.
///
/// This is the new primary type introduced in [MSC3552] and should not be sent before the end of
/// the transition period. See the documentation of the [`message`] module for more information.
///
/// `ImageEventContent` can be converted to a [`RoomMessageEventContent`] with a
/// [`MessageType::Image`]. You can convert it back with
/// [`ImageEventContent::from_image_room_message()`].
///
/// [MSC3552]: https://github.com/matrix-org/matrix-spec-proposals/pull/3552
/// [`message`]: super::message
/// [`RoomMessageEventContent`]: super::room::message::RoomMessageEventContent
/// [`MessageType::Image`]: super::room::message::MessageType::Image
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.image", kind = MessageLike)]
pub struct ImageEventContent {
    /// The text representation of the message.
    #[serde(flatten)]
    pub message: MessageContent,

    /// The file content of the message.
    #[serde(rename = "m.file")]
    pub file: FileContent,

    /// The image content of the message.
    #[serde(rename = "m.image")]
    pub image: Box<ImageContent>,

    /// The thumbnails of the message.
    #[serde(rename = "m.thumbnail", default, skip_serializing_if = "Vec::is_empty")]
    pub thumbnail: Vec<ThumbnailContent>,

    /// The captions of the message.
    #[serde(
        rename = "m.caption",
        with = "super::message::content_serde::as_vec",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub caption: Option<MessageContent>,

    /// Information about related messages.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Relation>,
}

impl ImageEventContent {
    /// Creates a new `ImageEventContent` with the given plain text message and file.
    pub fn plain(message: impl Into<String>, file: FileContent) -> Self {
        Self {
            message: MessageContent::plain(message),
            file,
            image: Default::default(),
            thumbnail: Default::default(),
            caption: Default::default(),
            relates_to: None,
        }
    }

    /// Creates a new non-encrypted `ImageEventContent` with the given message and file.
    pub fn with_message(message: MessageContent, file: FileContent) -> Self {
        Self {
            message,
            file,
            image: Default::default(),
            thumbnail: Default::default(),
            caption: Default::default(),
            relates_to: None,
        }
    }

    /// Create a new `ImageEventContent` from the given `ImageMessageEventContent` and optional
    /// relation.
    pub fn from_image_room_message(
        content: ImageMessageEventContent,
        relates_to: Option<Relation>,
    ) -> Self {
        let ImageMessageEventContent {
            body,
            source,
            info,
            message,
            file,
            image,
            thumbnail,
            caption,
        } = content;

        let message = message.unwrap_or_else(|| MessageContent::plain(body));
        let file = file.unwrap_or_else(|| {
            FileContent::from_room_message_content(source, info.as_deref(), None)
        });
        let image =
            image.or_else(|| info.as_deref().map(|info| Box::new(info.into()))).unwrap_or_default();
        let thumbnail = thumbnail
            .or_else(|| {
                info.as_deref()
                    .and_then(|info| {
                        ThumbnailContent::from_room_message_content(
                            info.thumbnail_source.as_ref(),
                            info.thumbnail_info.as_deref(),
                        )
                    })
                    .map(|thumbnail| vec![thumbnail])
            })
            .unwrap_or_default();

        Self { message, file, image, thumbnail, caption, relates_to }
    }
}

/// Image content.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ImageContent {
    /// The height of the image in pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<UInt>,

    /// The width of the image in pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<UInt>,
}

impl ImageContent {
    /// Creates a new empty `ImageContent`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `ImageContent` with the given width and height.
    pub fn with_size(width: UInt, height: UInt) -> Self {
        Self { height: Some(height), width: Some(width) }
    }

    /// Whether this `ImageContent` is empty.
    pub fn is_empty(&self) -> bool {
        self.height.is_none() && self.width.is_none()
    }
}

impl From<&ImageInfo> for ImageContent {
    fn from(info: &ImageInfo) -> Self {
        let ImageInfo { height, width, .. } = info;
        Self { height: height.to_owned(), width: width.to_owned() }
    }
}

impl From<&ThumbnailInfo> for ImageContent {
    fn from(info: &ThumbnailInfo) -> Self {
        let ThumbnailInfo { height, width, .. } = info;
        Self { height: height.to_owned(), width: width.to_owned() }
    }
}

/// Thumbnail content.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ThumbnailContent {
    /// The file info of the thumbnail.
    #[serde(flatten)]
    pub file: ThumbnailFileContent,

    /// The image info of the thumbnail.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub image: Option<Box<ImageContent>>,
}

impl ThumbnailContent {
    /// Creates a `ThumbnailContent` with the given file and image info.
    pub fn new(file: ThumbnailFileContent, image: Option<Box<ImageContent>>) -> Self {
        Self { file, image }
    }

    /// Create a `ThumbnailContent` with the given thumbnail source and info.
    ///
    /// Returns `None` if no thumbnail was found.
    pub fn from_room_message_content(
        thumbnail_source: Option<&MediaSource>,
        thumbnail_info: Option<&ThumbnailInfo>,
    ) -> Option<Self> {
        thumbnail_source.map(|thumbnail_source| {
            let file =
                ThumbnailFileContent::from_room_message_content(thumbnail_source, thumbnail_info);
            let image = thumbnail_info.map(|info| Box::new(info.into()));

            Self { file, image }
        })
    }
}

/// Thumbnail file content.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ThumbnailFileContent {
    /// The URL to the thumbnail.
    pub url: OwnedMxcUri,

    /// Information about the uploaded thumbnail.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<ThumbnailFileContentInfo>>,

    /// Information on the encrypted thumbnail.
    ///
    /// Required if the thumbnail is encrypted.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub encryption_info: Option<Box<EncryptedContent>>,
}

impl ThumbnailFileContent {
    /// Creates a new non-encrypted `ThumbnailFileContent` with the given url and file info.
    pub fn plain(url: OwnedMxcUri, info: Option<Box<ThumbnailFileContentInfo>>) -> Self {
        Self { url, info, encryption_info: None }
    }

    /// Creates a new encrypted `ThumbnailFileContent` with the given url, encryption info and
    /// thumbnail file info.
    pub fn encrypted(
        url: OwnedMxcUri,
        encryption_info: EncryptedContent,
        info: Option<Box<ThumbnailFileContentInfo>>,
    ) -> Self {
        Self { url, info, encryption_info: Some(Box::new(encryption_info)) }
    }

    /// Create a `ThumbnailFileContent` with the given thumbnail source and info.
    ///
    /// Returns `None` if no thumbnail was found.
    fn from_room_message_content(
        thumbnail_source: &MediaSource,
        thumbnail_info: Option<&ThumbnailInfo>,
    ) -> Self {
        match thumbnail_source {
            MediaSource::Plain(url) => {
                Self::plain(url.to_owned(), thumbnail_info.map(|info| Box::new(info.into())))
            }
            MediaSource::Encrypted(file) => Self::encrypted(
                file.url.clone(),
                (&**file).into(),
                thumbnail_info.map(|info| Box::new(info.into())),
            ),
        }
    }

    /// Whether the thumbnail file is encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.encryption_info.is_some()
    }
}

/// Information about a thumbnail file content.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ThumbnailFileContentInfo {
    /// The mimetype of the thumbnail, e.g. `image/png`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// The size of the thumbnail in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<UInt>,
}

impl ThumbnailFileContentInfo {
    /// Creates an empty `ThumbnailFileContentInfo`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl From<&ThumbnailInfo> for ThumbnailFileContentInfo {
    fn from(info: &ThumbnailInfo) -> Self {
        let ThumbnailInfo { mimetype, size, .. } = info;
        Self { mimetype: mimetype.to_owned(), size: size.to_owned() }
    }
}
