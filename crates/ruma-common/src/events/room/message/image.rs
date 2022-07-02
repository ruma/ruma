use serde::{Deserialize, Serialize};

#[cfg(feature = "unstable-msc3552")]
use crate::events::{
    file::{FileContent, FileContentInfo},
    image::{ImageContent, ThumbnailContent},
    message::MessageContent,
};
use crate::{
    events::room::{EncryptedFile, ImageInfo, MediaSource},
    OwnedMxcUri,
};

/// The payload for an image message.
///
/// With the `unstable-msc3552` feature, this type contains the transitional format of
/// [`ImageEventContent`]. See the documentation of the [`message`] module for more information.
///
/// [`ImageEventContent`]: crate::events::image::ImageEventContent
/// [`message`]: crate::events::message
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "msgtype", rename = "m.image")]
#[cfg_attr(
    feature = "unstable-msc3552",
    serde(from = "super::content_serde::ImageMessageEventContentDeHelper")
)]
pub struct ImageMessageEventContent {
    /// A textual representation of the image.
    ///
    /// Could be the alt text of the image, the filename of the image, or some kind of content
    /// description for accessibility e.g. "image attachment".
    pub body: String,

    /// The source of the image.
    #[serde(flatten)]
    pub source: MediaSource,

    /// Metadata about the image referred to in `source`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<ImageInfo>>,

    /// Extensible-event text representation of the message.
    ///
    /// If present, this should be preferred over the `body` field.
    #[cfg(feature = "unstable-msc3552")]
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub message: Option<MessageContent>,

    /// Extensible-event file content of the message.
    ///
    /// If present, this should be preferred over the `source` and `info` fields.
    #[cfg(feature = "unstable-msc3552")]
    #[serde(rename = "org.matrix.msc1767.file", skip_serializing_if = "Option::is_none")]
    pub file: Option<FileContent>,

    /// Extensible-event image info of the message.
    ///
    /// If present, this should be preferred over the `info` field.
    #[cfg(feature = "unstable-msc3552")]
    #[serde(rename = "org.matrix.msc1767.image", skip_serializing_if = "Option::is_none")]
    pub image: Option<Box<ImageContent>>,

    /// Extensible-event thumbnails of the message.
    ///
    /// If present, this should be preferred over the `info` field.
    #[cfg(feature = "unstable-msc3552")]
    #[serde(rename = "org.matrix.msc1767.thumbnail", skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<Vec<ThumbnailContent>>,

    /// Extensible-event captions of the message.
    #[cfg(feature = "unstable-msc3552")]
    #[serde(
        rename = "org.matrix.msc1767.caption",
        with = "crate::events::message::content_serde::as_vec",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub caption: Option<MessageContent>,
}

impl ImageMessageEventContent {
    /// Creates a new non-encrypted `ImageMessageEventContent` with the given body, url and
    /// optional extra info.
    pub fn plain(body: String, url: OwnedMxcUri, info: Option<Box<ImageInfo>>) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3552")]
            message: Some(MessageContent::plain(body.clone())),
            #[cfg(feature = "unstable-msc3552")]
            file: Some(FileContent::plain(
                url.clone(),
                info.as_deref().and_then(|info| {
                    FileContentInfo::from_room_message_content(
                        None,
                        info.mimetype.to_owned(),
                        info.size,
                    )
                    .map(Box::new)
                }),
            )),
            #[cfg(feature = "unstable-msc3552")]
            image: Some(Box::new(
                info.as_deref()
                    .and_then(|info| {
                        ImageContent::from_room_message_content(info.width, info.height)
                    })
                    .unwrap_or_default(),
            )),
            #[cfg(feature = "unstable-msc3552")]
            thumbnail: info
                .as_deref()
                .and_then(|info| {
                    ThumbnailContent::from_room_message_content(
                        info.thumbnail_source.to_owned(),
                        info.thumbnail_info.to_owned(),
                    )
                })
                .map(|thumbnail| vec![thumbnail]),
            #[cfg(feature = "unstable-msc3552")]
            caption: None,
            body,
            source: MediaSource::Plain(url),
            info,
        }
    }

    /// Creates a new encrypted `ImageMessageEventContent` with the given body and encrypted
    /// file.
    pub fn encrypted(body: String, file: EncryptedFile) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3552")]
            message: Some(MessageContent::plain(body.clone())),
            #[cfg(feature = "unstable-msc3552")]
            file: Some(FileContent::encrypted(file.url.clone(), (&file).into(), None)),
            #[cfg(feature = "unstable-msc3552")]
            image: Some(Box::new(ImageContent::default())),
            #[cfg(feature = "unstable-msc3552")]
            thumbnail: None,
            #[cfg(feature = "unstable-msc3552")]
            caption: None,
            body,
            source: MediaSource::Encrypted(Box::new(file)),
            info: None,
        }
    }

    /// Create a new `ImageMessageEventContent` with the given message, file info, image info,
    /// thumbnails and captions.
    #[cfg(feature = "unstable-msc3552")]
    pub fn from_extensible_content(
        message: MessageContent,
        file: FileContent,
        image: Box<ImageContent>,
        thumbnail: Vec<ThumbnailContent>,
        caption: Option<MessageContent>,
    ) -> Self {
        let body = if let Some(body) = message.find_plain() {
            body.to_owned()
        } else {
            message[0].body.clone()
        };
        let source = (&file).into();
        let info = ImageInfo::from_extensible_content(file.info.as_deref(), &image, &thumbnail)
            .map(Box::new);
        let thumbnail = if thumbnail.is_empty() { None } else { Some(thumbnail) };

        Self {
            message: Some(message),
            file: Some(file),
            image: Some(image),
            thumbnail,
            caption,
            body,
            source,
            info,
        }
    }
}
