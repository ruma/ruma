//! Types for [`m.room.image_pack`] event.
//!
//! [`m.room.image_pack`]: https://spec.matrix.org/v1.19/client-server-api/#mroomimage_pack

use std::collections::{BTreeMap, BTreeSet};

use ruma_common::OwnedMxcUri;
use ruma_macros::{EventContent, StringEnum};
use serde::{Deserialize, Serialize};

use crate::{PrivOwnedStr, room::ImageInfo};

/// The content of an [`m.room.image_pack`] event.
///
/// The state key is the unique identifier for the image pack.
///
/// [`m.room.image_pack`]: https://spec.matrix.org/v1.19/client-server-api/#mroomimage_pack
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.room.image_pack", kind = State, state_key_type = String)]
pub struct RoomImagePackEventContent {
    /// A map from a shortcode to an image object.
    ///
    /// Each entry defines one image available in this pack.
    pub images: BTreeMap<String, ImagePackImage>,

    /// Metadata about the image pack as a whole.
    ///
    /// This field is not serialized if it is empty, and deserializes to its default value if it is
    /// missing.
    #[serde(default, skip_serializing_if = "ImagePackMeta::is_empty")]
    pub pack: ImagePackMeta,
}

impl RoomImagePackEventContent {
    /// Creates a new `RoomImagePackEventContent` with a list of images.
    pub fn new(images: BTreeMap<String, ImagePackImage>) -> Self {
        Self { images, pack: ImagePackMeta::default() }
    }
}

/// An image object in an image pack.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct ImagePackImage {
    /// The MXC URI to the media file.
    pub url: OwnedMxcUri,

    /// An optional text body for this image.
    ///
    /// Useful for the sticker body text or the emote alt text.
    ///
    /// Defaults to the shortcode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// The [ImageInfo] object used for the `info` block of `m.sticker` events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<ImageInfo>,
}

impl ImagePackImage {
    /// Creates a new `ImagePackImage` with the given MXC URI to the media file.
    pub fn new(url: OwnedMxcUri) -> Self {
        Self { url, body: None, info: None }
    }
}

/// Details about an image pack.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct ImagePackMeta {
    /// A display name for the pack.
    ///
    /// If absent and the pack is defined in a room, defaults to the room's name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The MXC URI of an avatar for the pack.
    ///
    /// If absent and the pack is defined in a room, defaults to the room's avatar.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<OwnedMxcUri>,

    /// The intended usage(s) for this pack.
    ///
    /// If empty, all usage types are assumed.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub usage: BTreeSet<PackUsage>,

    /// The attribution of this pack.
    ///
    /// For crediting the original author or source, for example.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,
}

impl ImagePackMeta {
    /// Creates a new empty `ImagePackMeta`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether this `ImagePackMeta` is empty.
    fn is_empty(&self) -> bool {
        let Self { display_name, avatar_url, usage, attribution } = self;
        display_name.is_none() && avatar_url.is_none() && usage.is_empty() && attribution.is_none()
    }
}

/// The intended usages for an image pack.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PackUsage {
    /// The images are intended to be sent inline in messages.
    Emoticon,

    /// The images are intended to be sent as standalone sticker events.
    Sticker,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
