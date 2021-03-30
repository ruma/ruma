//! Types for the *m.room.avatar* event.

use js_int::UInt;
use ruma_events_macros::StateEventContent;
use serde::{Deserialize, Serialize};

use super::ThumbnailInfo;
use crate::StateEvent;

/// A picture that is associated with the room.
///
/// This can be displayed alongside the room information.
pub type AvatarEvent = StateEvent<AvatarEventContent>;

/// The payload for `AvatarEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[cfg_attr(feature = "unstable-pre-spec", derive(Default))]
#[ruma_event(type = "m.room.avatar")]
pub struct AvatarEventContent {
    /// Information about the avatar image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<ImageInfo>>,

    /// URL of the avatar image.
    #[cfg(not(feature = "unstable-pre-spec"))]
    pub url: String,

    /// URL of the avatar image.
    #[cfg(feature = "unstable-pre-spec")]
    pub url: Option<String>,
}

impl AvatarEventContent {
    /// Create an `AvatarEventContent` from the given image URL.
    #[cfg(not(feature = "unstable-pre-spec"))]
    pub fn new(url: String) -> Self {
        Self { info: None, url }
    }

    /// Create an empty `AvatarEventContent`.
    #[cfg(feature = "unstable-pre-spec")]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Metadata about an image (specific to avatars).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ImageInfo {
    /// The height of the image in pixels.
    #[serde(rename = "h", skip_serializing_if = "Option::is_none")]
    pub height: Option<UInt>,

    /// The width of the image in pixels.
    #[serde(rename = "w", skip_serializing_if = "Option::is_none")]
    pub width: Option<UInt>,

    /// The MIME type of the image, e.g. "image/png."
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// The file size of the image in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<UInt>,

    /// Metadata about the image referred to in `thumbnail_url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_info: Option<Box<ThumbnailInfo>>,

    /// The URL to the thumbnail of the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,

    /// The [BlurHash](https://blurha.sh) for this image.
    ///
    /// This uses the unstable prefix in
    /// [MSC2448](https://github.com/matrix-org/matrix-doc/pull/2448).
    #[cfg(feature = "unstable-pre-spec")]
    #[serde(rename = "xyz.amorgan.blurhash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blurhash: Option<String>,
}
