//! Types for extensible location message events ([MSC3488]).
//!
//! [MSC3488]: https://github.com/matrix-org/matrix-spec-proposals/pull/3488

use js_int::UInt;
use ruma_macros::{EventContent, StringEnum};
use serde::{Deserialize, Serialize};

mod zoomlevel_serde;

use super::{
    message::MessageContent,
    room::message::{LocationMessageEventContent, Relation},
};
use crate::{MilliSecondsSinceUnixEpoch, PrivOwnedStr};

/// The payload for an extensible location message.
///
/// This is the new primary type introduced in [MSC3488] and should not be sent before the end of
/// the transition period. See the documentation of the [`message`] module for more information.
///
/// `LocationEventContent` can be converted to a [`RoomMessageEventContent`] with a
/// [`MessageType::Location`]. You can convert it back with
/// [`LocationEventContent::from_location_room_message()`].
///
/// [MSC3488]: https://github.com/matrix-org/matrix-spec-proposals/pull/3488
/// [`message`]: super::message
/// [`RoomMessageEventContent`]: super::room::message::RoomMessageEventContent
/// [`MessageType::Location`]: super::room::message::MessageType::Location
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.location", kind = MessageLike)]
pub struct LocationEventContent {
    /// The text representation of the message.
    #[serde(flatten)]
    pub message: MessageContent,

    /// The location info of the message.
    #[serde(rename = "m.location")]
    pub location: LocationContent,

    /// The asset this message refers to.
    #[serde(default, rename = "m.asset", skip_serializing_if = "ruma_common::serde::is_default")]
    pub asset: AssetContent,

    /// The timestamp this message refers to.
    #[serde(rename = "m.ts", skip_serializing_if = "Option::is_none")]
    pub ts: Option<MilliSecondsSinceUnixEpoch>,

    /// Information about related messages.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Relation>,
}

impl LocationEventContent {
    /// Creates a new `LocationEventContent` with the given plain text representation and location.
    pub fn plain(message: impl Into<String>, location: LocationContent) -> Self {
        Self {
            message: MessageContent::plain(message),
            location,
            asset: Default::default(),
            ts: None,
            relates_to: None,
        }
    }

    /// Creates a new `LocationEventContent` with the given text representation and location.
    pub fn with_message(message: MessageContent, location: LocationContent) -> Self {
        Self { message, location, asset: Default::default(), ts: None, relates_to: None }
    }

    /// Create a new `LocationEventContent` from the given `LocationMessageEventContent` and
    /// optional relation.
    pub fn from_location_room_message(
        content: LocationMessageEventContent,
        relates_to: Option<Relation>,
    ) -> Self {
        let LocationMessageEventContent { body, geo_uri, message, location, asset, ts, .. } =
            content;

        let message = message.unwrap_or_else(|| MessageContent::plain(body));
        let location = location.unwrap_or_else(|| LocationContent::new(geo_uri));
        let asset = asset.unwrap_or_default();

        Self { message, location, asset, ts, relates_to }
    }
}

/// Location content.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct LocationContent {
    /// A `geo:` URI representing the location.
    ///
    /// See [RFC 5870](https://datatracker.ietf.org/doc/html/rfc5870) for more details.
    pub uri: String,

    /// The description of the location.
    ///
    /// It should be used to label the location on a map.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A zoom level to specify the displayed area size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zoom_level: Option<ZoomLevel>,
}

impl LocationContent {
    /// Creates a new `LocationContent` with the given geo URI.
    pub fn new(uri: String) -> Self {
        Self { uri, description: None, zoom_level: None }
    }
}

/// An error encountered when trying to convert to a `ZoomLevel`.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum ZoomLevelError {
    /// The value is higher than [`ZoomLevel::MAX`].
    #[error("value too high")]
    TooHigh,
}

/// A zoom level.
///
/// This is an integer between 0 and 20 as defined in the [OpenStreetMap Wiki].
///
/// [OpenStreetMap Wiki]: https://wiki.openstreetmap.org/wiki/Zoom_levels
#[derive(Clone, Debug, Serialize)]
pub struct ZoomLevel(UInt);

impl ZoomLevel {
    /// The smallest value of a `ZoomLevel`, 0.
    pub const MIN: u8 = 0;

    /// The largest value of a `ZoomLevel`, 20.
    pub const MAX: u8 = 20;

    /// Creates a new `ZoomLevel` with the given value.
    pub fn new(value: u8) -> Option<Self> {
        if value > Self::MAX {
            None
        } else {
            Some(Self(value.into()))
        }
    }

    /// The value of this `ZoomLevel`.
    pub fn get(&self) -> UInt {
        self.0
    }
}

impl TryFrom<u8> for ZoomLevel {
    type Error = ZoomLevelError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(ZoomLevelError::TooHigh)
    }
}

/// Asset content.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct AssetContent {
    /// The type of asset being referred to.
    #[serde(rename = "type")]
    pub type_: AssetType,
}

impl AssetContent {
    /// Creates a new default `AssetContent`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// The type of an asset.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, StringEnum)]
#[non_exhaustive]
pub enum AssetType {
    /// The asset is the sender of the event.
    #[ruma_enum(rename = "m.self")]
    Self_,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl Default for AssetType {
    fn default() -> Self {
        Self::Self_
    }
}
