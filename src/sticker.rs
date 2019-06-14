//! Types for the *m.sticker* event.

use serde::{Deserialize, Serialize};

use crate::room::ImageInfo;

room_event! {
    /// A sticker message.
    pub struct StickerEvent(StickerEventContent) {}
}

/// The payload of a `StickerEvent`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct StickerEventContent {
    /// A textual representation or associated description of the sticker image. This could be the
    /// alt text of the original image, or a message to accompany and further describe the sticker.
    pub body: String,

    /// Metadata about the image referred to in `url` including a thumbnail representation.
    pub info: ImageInfo,

    /// The URL to the sticker image. This must be a valid `mxc://` URI.
    pub url: String,
}
