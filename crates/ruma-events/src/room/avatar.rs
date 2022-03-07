//! Types for the [`m.room.avatar`] event.
//!
//! [`m.room.avatar`]: https://spec.matrix.org/v1.2/client-server-api/#mroomavatar

use js_int::UInt;
use ruma_events_macros::EventContent;
use ruma_identifiers::MxcUri;
use serde::{Deserialize, Serialize};

use super::ThumbnailInfo;

/// The content of an `m.room.avatar` event.
///
/// A picture that is associated with the room.
///
/// This can be displayed alongside the room information.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[cfg_attr(feature = "unstable-pre-spec", derive(Default))]
#[ruma_event(type = "m.room.avatar", kind = State)]
pub struct RoomAvatarEventContent {
    /// Information about the avatar image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Box<ImageInfo>>,

    /// URL of the avatar image.
    ///
    /// With the `unstable-pre-spec` feature, this field is optional.
    /// See [matrix-doc#2006](https://github.com/matrix-org/matrix-doc/issues/2006).
    #[cfg(not(feature = "unstable-pre-spec"))]
    pub url: Box<MxcUri>,

    /// URL of the avatar image.
    ///
    /// Without the `unstable-pre-spec` feature, this field is not optional.
    /// See [matrix-doc#2006](https://github.com/matrix-org/matrix-doc/issues/2006).
    #[cfg(feature = "unstable-pre-spec")]
    pub url: Option<Box<MxcUri>>,
}

impl RoomAvatarEventContent {
    /// Create an `RoomAvatarEventContent` from the given image URL.
    ///
    /// With the `unstable-pre-spec` feature, this method takes no parameters.
    /// See [matrix-doc#2006](https://github.com/matrix-org/matrix-doc/issues/2006).
    #[cfg(not(feature = "unstable-pre-spec"))]
    pub fn new(url: Box<MxcUri>) -> Self {
        Self { info: None, url }
    }

    /// Create an empty `RoomAvatarEventContent`.
    ///
    /// With the `unstable-pre-spec` feature, this method takes an `MxcUri`.
    /// See [matrix-doc#2006](https://github.com/matrix-org/matrix-doc/issues/2006).
    #[cfg(feature = "unstable-pre-spec")]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Metadata about an image (specific to avatars).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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
    pub thumbnail_url: Option<Box<MxcUri>>,

    /// The [BlurHash](https://blurha.sh) for this image.
    ///
    /// This uses the unstable prefix in
    /// [MSC2448](https://github.com/matrix-org/matrix-doc/pull/2448).
    #[cfg(feature = "unstable-msc2448")]
    #[serde(rename = "xyz.amorgan.blurhash", skip_serializing_if = "Option::is_none")]
    pub blurhash: Option<String>,
}

impl ImageInfo {
    /// Create a new `ImageInfo` with all fields set to `None`.
    pub fn new() -> Self {
        Self::default()
    }
}
