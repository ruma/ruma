//! Types for the [`m.sticker`] event.
//!
//! [`m.sticker`]: https://spec.matrix.org/latest/client-server-api/#msticker

use ruma_common::OwnedMxcUri;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::room::ImageInfo;

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

    /// The URL to the sticker image.
    pub url: OwnedMxcUri,
}

impl StickerEventContent {
    /// Creates a new `StickerEventContent` with the given body, image info and URL.
    pub fn new(body: String, info: ImageInfo, url: OwnedMxcUri) -> Self {
        Self { body, info, url }
    }
}
