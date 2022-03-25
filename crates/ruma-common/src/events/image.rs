//! Types for extensible image message events ([MSC3552]).
//!
//! [MSC3552]: https://github.com/matrix-org/matrix-spec-proposals/pull/3552

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{
    file::{EncryptedContent, FileContent},
    message::{MessageContent, Text},
    room::message::Relation,
};
use crate::MxcUri;

/// The payload for an extensible image message.
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
    #[serde(rename = "m.thumbnail", default, skip_serializing_if = "Thumbnails::is_empty")]
    pub thumbnail: Thumbnails,

    /// The captions of the message.
    #[serde(rename = "m.caption", default, skip_serializing_if = "Captions::is_empty")]
    pub caption: Captions,

    /// Information about related messages for [rich replies].
    ///
    /// [rich replies]: https://spec.matrix.org/v1.2/client-server-api/#rich-replies
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

/// Thumbnail file content.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ThumbnailFileContent {
    /// The URL to the thumbnail.
    pub url: Box<MxcUri>,

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
    pub fn plain(url: Box<MxcUri>, info: Option<Box<ThumbnailFileContentInfo>>) -> Self {
        Self { url, info, encryption_info: None }
    }

    /// Creates a new encrypted `ThumbnailFileContent` with the given url, encryption info and
    /// thumbnail file info.
    pub fn encrypted(
        url: Box<MxcUri>,
        encryption_info: EncryptedContent,
        info: Option<Box<ThumbnailFileContentInfo>>,
    ) -> Self {
        Self { url, info, encryption_info: Some(Box::new(encryption_info)) }
    }

    /// Whether the thumbnail file is encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.encryption_info.is_some()
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
}

/// An array of thumbnails.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Thumbnails(pub(crate) Vec<ThumbnailContent>);

impl Thumbnails {
    /// Creates a new `Thumbnails` with the given thumbnails.
    ///
    /// The thumbnails must be ordered by most preferred first.
    pub fn new(thumbnails: &[ThumbnailContent]) -> Self {
        Self(thumbnails.to_owned())
    }

    /// Get the thumbnails.
    ///
    /// The thumbnails are ordered by most preferred first.
    pub fn thumbnails(&self) -> &[ThumbnailContent] {
        &self.0
    }

    /// Whether this is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// An array of captions.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Captions(pub(crate) Vec<Text>);

impl Captions {
    /// Creates a new `Captions` with the given captions.
    ///
    /// The captions must be ordered by most preferred first.
    pub fn new(captions: &[Text]) -> Self {
        Self(captions.to_owned())
    }

    /// A convenience constructor to create a plain text caption.
    pub fn plain(body: impl Into<String>) -> Self {
        Self(vec![Text::plain(body)])
    }

    /// A convenience constructor to create an HTML caption.
    pub fn html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self(vec![Text::html(html_body), Text::plain(body)])
    }

    /// A convenience constructor to create a Markdown caption.
    ///
    /// Returns an HTML caption if some Markdown formatting was detected, otherwise returns a plain
    /// text caption.
    #[cfg(feature = "markdown")]
    pub fn markdown(body: impl AsRef<str> + Into<String>) -> Self {
        let mut message = Vec::with_capacity(2);
        if let Some(html_body) = Text::markdown(&body) {
            message.push(html_body);
        }
        message.push(Text::plain(body));
        Self(message)
    }

    /// Get the captions.
    ///
    /// The captions are ordered by most preferred first.
    pub fn captions(&self) -> &[Text] {
        &self.0
    }

    /// Whether this is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the plain text representation of this caption.
    pub fn find_plain(&self) -> Option<&str> {
        self.captions()
            .iter()
            .find(|content| content.mimetype == "text/plain")
            .map(|content| content.body.as_ref())
    }

    /// Get the HTML representation of this caption.
    pub fn find_html(&self) -> Option<&str> {
        self.captions()
            .iter()
            .find(|content| content.mimetype == "text/html")
            .map(|content| content.body.as_ref())
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
}
