//! Types for image packs in Matrix ([MSC2545]).
//!
//! [MSC2545]: https://github.com/matrix-org/matrix-spec-proposals/pull/2545

use std::collections::{BTreeMap, BTreeSet};

use ruma_common::{serde::StringEnum, OwnedMxcUri, OwnedRoomId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{room::ImageInfo, PrivOwnedStr};

/// The content of an `im.ponies.room_emotes` event,
/// the unstable version of `m.image_pack` in room state events.
///
/// State key is the identifier for the image pack in [ImagePackRoomsEventContent].
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "im.ponies.room_emotes", kind = State, state_key_type = String)]
pub struct RoomImagePackEventContent {
    /// A list of images available in this image pack.
    ///
    /// Keys in the map are shortcodes for the images.
    pub images: BTreeMap<String, PackImage>,

    /// Image pack info.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack: Option<PackInfo>,
}

impl RoomImagePackEventContent {
    /// Creates a new `RoomImagePackEventContent` with a list of images.
    pub fn new(images: BTreeMap<String, PackImage>) -> Self {
        Self { images, pack: None }
    }
}

/// The content of an `im.ponies.user_emotes` event,
/// the unstable version of `m.image_pack` in account data events.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "im.ponies.user_emotes", kind = GlobalAccountData)]
pub struct AccountImagePackEventContent {
    /// A list of images available in this image pack.
    ///
    /// Keys in the map are shortcodes for the images.
    pub images: BTreeMap<String, PackImage>,

    /// Image pack info.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack: Option<PackInfo>,
}

impl AccountImagePackEventContent {
    /// Creates a new `AccountImagePackEventContent` with a list of images.
    pub fn new(images: BTreeMap<String, PackImage>) -> Self {
        Self { images, pack: None }
    }
}

/// An image object in a image pack.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct PackImage {
    /// The MXC URI to the media file.
    pub url: OwnedMxcUri,

    /// An optional text body for this image.
    /// Useful for the sticker body text or the emote alt text.
    ///
    /// Defaults to the shortcode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// The [ImageInfo] object used for the `info` block of `m.sticker` events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<ImageInfo>,

    /// The usages for the image.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub usage: BTreeSet<PackUsage>,
}

impl PackImage {
    /// Creates a new `PackImage` with the given MXC URI to the media file.
    pub fn new(url: OwnedMxcUri) -> Self {
        Self { url, body: None, info: None, usage: BTreeSet::new() }
    }
}

/// A description for the pack.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct PackInfo {
    /// A display name for the pack.
    /// This does not have to be unique from other packs in a room.
    ///
    /// Defaults to the room name, if the image pack event is in the room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The MXC URI of an avatar/icon to display for the pack.
    ///
    /// Defaults to the room avatar, if the pack is in the room.
    /// Otherwise, the pack does not have an avatar.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<OwnedMxcUri>,

    /// The usages for the pack.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub usage: BTreeSet<PackUsage>,

    /// The attribution of this pack.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,
}

impl PackInfo {
    /// Creates a new empty `PackInfo`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Usages for either an image pack or an individual image.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PackUsage {
    /// Pack or image is usable as a emoticon.
    Emoticon,

    /// Pack or image is usable as a sticker.
    Sticker,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The content of an `im.ponies.emote_rooms` event,
/// the unstable version of `m.image_pack.rooms`.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "im.ponies.emote_rooms", kind = GlobalAccountData)]
pub struct ImagePackRoomsEventContent {
    /// A map of enabled image packs in each room.
    pub rooms: BTreeMap<OwnedRoomId, BTreeMap<String, ImagePackRoomContent>>,
}

impl ImagePackRoomsEventContent {
    /// Creates a new `ImagePackRoomsEventContent`
    /// with a map of enabled image packs in each room.
    pub fn new(rooms: BTreeMap<OwnedRoomId, BTreeMap<String, ImagePackRoomContent>>) -> Self {
        Self { rooms }
    }
}

/// Additional metadatas for a enabled room image pack.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct ImagePackRoomContent {}

impl ImagePackRoomContent {
    /// Creates a new empty `ImagePackRoomContent`.
    pub fn new() -> Self {
        Self {}
    }
}
