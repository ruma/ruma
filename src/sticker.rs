//! Types for the *m.sticker* event.

use ruma_events_macros::{FromRaw, MessageEventContent};
use serde::Serialize;

use crate::room::ImageInfo;

/// A sticker message.
#[derive(Clone, Debug, Serialize, FromRaw, MessageEventContent)]
#[ruma_event(type = "m.sticker")]
pub struct StickerEventContent {
    /// A textual representation or associated description of the sticker image. This could
    /// be the alt text of the original image, or a message to accompany and further
    /// describe the sticker.
    pub body: String,

    /// Metadata about the image referred to in `url` including a thumbnail representation.
    pub info: ImageInfo,

    /// The URL to the sticker image. This must be a valid `mxc://` URI.
    pub url: String,
}
