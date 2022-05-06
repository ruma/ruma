//! Types for the [`m.sticker`] event.
//!
//! [`m.sticker`]: https://spec.matrix.org/v1.2/client-server-api/#msticker

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

#[cfg(feature = "unstable-msc3552")]
use super::{
    file::FileContent,
    image::{ImageContent, ThumbnailContent},
    message::MessageContent,
};
use crate::{events::room::ImageInfo, OwnedMxcUri};

/// The content of an `m.sticker` event.
///
/// A sticker message.
///
/// With the `unstable-msc3552` feature, this type also contains the transitional extensible events
/// format. See the documentation of the [`message`] module for more information.
///
/// [`message`]: super::message
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

    /// The URL to the sticker image.
    pub url: OwnedMxcUri,

    /// Extensible-event text representation of the message.
    ///
    /// If present, this should be preferred over the `body` field.
    #[cfg(feature = "unstable-msc3552")]
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub message: Option<MessageContent>,

    /// Extensible-event file content of the message.
    ///
    /// If present, this should be preferred over the `url`, `file` and `info` fields.
    #[cfg(feature = "unstable-msc3552")]
    #[serde(
        rename = "org.matrix.msc1767.file",
        alias = "m.file",
        skip_serializing_if = "Option::is_none"
    )]
    pub file: Option<FileContent>,

    /// Extensible-event image info of the message.
    ///
    /// If present, this should be preferred over the `info` field.
    #[cfg(feature = "unstable-msc3552")]
    #[serde(
        rename = "org.matrix.msc1767.image",
        alias = "m.image",
        skip_serializing_if = "Option::is_none"
    )]
    pub image: Option<Box<ImageContent>>,

    /// Extensible-event thumbnails of the message.
    ///
    /// If present, this should be preferred over the `info` field.
    #[cfg(feature = "unstable-msc3552")]
    #[serde(
        rename = "org.matrix.msc1767.thumbnail",
        alias = "m.thumbnail",
        skip_serializing_if = "Option::is_none"
    )]
    pub thumbnail: Option<Vec<ThumbnailContent>>,

    /// Extensible-event captions of the message.
    #[cfg(feature = "unstable-msc3552")]
    #[serde(
        rename = "org.matrix.msc1767.caption",
        alias = "m.caption",
        with = "super::message::content_serde::as_vec",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub caption: Option<MessageContent>,
}

impl StickerEventContent {
    /// Creates a new `StickerEventContent` with the given body, image info and URL.
    pub fn new(body: String, info: ImageInfo, url: OwnedMxcUri) -> Self {
        Self {
            #[cfg(feature = "unstable-msc3552")]
            message: Some(MessageContent::plain(body.clone())),
            #[cfg(feature = "unstable-msc3552")]
            file: Some(FileContent::plain(url.clone(), Some(Box::new((&info).into())))),
            #[cfg(feature = "unstable-msc3552")]
            image: Some(Box::new((&info).into())),
            #[cfg(feature = "unstable-msc3552")]
            thumbnail: ThumbnailContent::from_room_message_content(
                info.thumbnail_source.as_ref(),
                info.thumbnail_info.as_deref(),
            )
            .map(|thumbnail| vec![thumbnail]),
            #[cfg(feature = "unstable-msc3552")]
            caption: None,
            body,
            info,
            url,
        }
    }
}
